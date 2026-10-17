//! Loading external references: finding the referenced drawing, reading it through the host's io
//! hooks, and bringing its model space into the xref's block definition with its symbols renamed
//! `xref|name`.
//!
//! A path is looked for as given (relative paths against the host drawing's folder), then by its
//! file name in the host drawing's folder. A referenced drawing's own attached xrefs load with it
//! (overlays don't), at most [`MAX_BLOCK_DEPTH`] levels deep; a drawing that is already being
//! loaded further up (a circular reference) is not loaded again. Every failure leaves the xref's
//! block empty with a status saying why; nothing here stops the host drawing from opening.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use cadcraft_doc::{Attrib, Drawing, Entity, EntityKind, EntityStore, MAX_BLOCK_DEPTH, XrefStatus};

/// Most objects the xrefs of one drawing load in one go (nested xrefs included).
pub(crate) const MAX_XREF_OBJECTS: usize = 4_000_000;
/// Most referenced files read in one go (nested xrefs included).
const MAX_XREF_FILES: usize = 1000;
/// Largest referenced file read.
#[cfg(not(target_arch = "wasm32"))]
const MAX_XREF_BYTES: u64 = 512 << 20;

/// The kinds of named symbols an object can refer to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum Sym {
    Layer,
    Linetype,
    Style,
    DimStyle,
    Block,
    MLeaderStyle,
    TableStyle,
}

/// Names no xref brings: layer 0 and the built-in linetypes stay the host's.
fn shared(sym: Sym, name: &str) -> bool {
    name.is_empty()
        || match sym {
            Sym::Layer => name == "0",
            Sym::Linetype => ["ByLayer", "ByBlock", "Continuous"].iter().any(|n| n.eq_ignore_ascii_case(name)),
            _ => false,
        }
}

/// Whether `name` starts with `prefix`, ignoring ASCII case.
pub(crate) fn has_prefix(name: &str, prefix: &str) -> bool {
    name.get(..prefix.len()).is_some_and(|p| p.eq_ignore_ascii_case(prefix))
}

/// Rename the symbols entity `e` refers to (`f` returns the new name, or `None` to keep it).
pub(crate) fn rename_refs(e: &mut Entity, f: &dyn Fn(Sym, &str) -> Option<String>) {
    let fix = |sym: Sym, n: &mut String| {
        if let Some(to) = f(sym, n) {
            *n = to;
        }
    };
    fix(Sym::Layer, &mut e.common.layer);
    fix(Sym::Linetype, &mut e.common.linetype);
    let attrib = |a: &mut Attrib| {
        fix(Sym::Style, &mut a.text.style);
        if let Some(l) = &mut a.props.layer {
            fix(Sym::Layer, l);
        }
        if let Some(l) = &mut a.props.linetype {
            fix(Sym::Linetype, l);
        }
    };
    match &mut e.kind {
        EntityKind::Text(t) => fix(Sym::Style, &mut t.style),
        EntityKind::MText(m) => fix(Sym::Style, &mut m.style),
        EntityKind::AttDef(a) => attrib(a),
        EntityKind::Insert(i) => {
            fix(Sym::Block, &mut i.block);
            i.attribs.iter_mut().for_each(attrib);
        }
        EntityKind::Dimension(dm) => {
            fix(Sym::DimStyle, &mut dm.style);
            if let Some(b) = &mut dm.block {
                fix(Sym::Block, b);
            }
        }
        EntityKind::Leader(l) => fix(Sym::DimStyle, &mut l.style),
        EntityKind::MLeader(m) => {
            fix(Sym::MLeaderStyle, &mut m.style);
            if let Some(t) = &mut m.text {
                fix(Sym::Style, &mut t.style);
            }
            if let Some(b) = &mut m.block {
                fix(Sym::Block, &mut b.block);
            }
        }
        EntityKind::Table(t) => fix(Sym::TableStyle, &mut t.style),
        _ => {}
    }
}

/// The outcome of loading one xref, for the command line.
#[derive(Clone, Debug)]
pub(crate) struct Outcome {
    pub name: String,
    pub status: XrefStatus,
    /// Why it isn't loaded, or the file it was loaded from.
    pub detail: String,
}

impl Outcome {
    pub fn message(&self) -> String {
        match self.status {
            XrefStatus::Loaded => format!("\"{}\" loaded from {}.", self.name, self.detail),
            _ => format!("Xref \"{}\" not loaded: {}.", self.name, self.detail),
        }
    }
}

/// What one load (with its nested xrefs) has used so far.
struct Ctx {
    /// The files being loaded, outermost first (a file already here is a circular reference).
    chain: Vec<PathBuf>,
    objects: usize,
    files: usize,
    /// Messages about nested xrefs.
    notes: Vec<String>,
}

/// The folder of the host drawing at `host` (a file path).
fn folder(host: Option<&str>) -> Option<PathBuf> {
    let p = Path::new(host?).parent()?;
    Some(if p.as_os_str().is_empty() { PathBuf::from(".") } else { p.to_path_buf() })
}

#[cfg(not(target_arch = "wasm32"))]
fn canonical(p: &Path) -> PathBuf {
    std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf())
}
#[cfg(target_arch = "wasm32")]
fn canonical(p: &Path) -> PathBuf {
    p.to_path_buf()
}

/// Where to look for a referenced drawing saved as `path`: the path itself (a relative one against
/// the host's folder), then its file name in the host's folder. Both separators work everywhere.
fn candidates(path: &str, dir: Option<&Path>) -> Vec<PathBuf> {
    let norm = if cfg!(windows) { path.to_string() } else { path.replace('\\', "/") };
    let p = PathBuf::from(norm.trim());
    let mut out = Vec::new();
    match dir {
        Some(d) if !p.is_absolute() => out.push(d.join(&p)),
        _ => out.push(p.clone()),
    }
    if let (Some(d), Some(f)) = (dir, p.file_name()) {
        out.push(d.join(f));
    }
    out
}

/// The file a referenced drawing saved as `path` is loaded from, if there is one.
pub(crate) fn find(path: &str, dir: Option<&Path>) -> Option<PathBuf> {
    if path.trim().is_empty() {
        return None;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        candidates(path, dir).into_iter().find(|p| p.is_file())
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = candidates(path, dir);
        None
    }
}

/// Read a drawing file through the host's io hooks.
fn read(file: &Path) -> Result<Drawing, String> {
    let hooks = super::super::super::file::io().ok_or("file formats are not available in this build")?;
    #[cfg(not(target_arch = "wasm32"))]
    {
        let len = std::fs::metadata(file).map_err(|e| e.to_string())?.len();
        if len > MAX_XREF_BYTES {
            return Err(format!("the file is larger than {} MB", MAX_XREF_BYTES >> 20));
        }
        let bytes = std::fs::read(file).map_err(|e| e.to_string())?;
        (hooks.read)(&bytes, &file.to_string_lossy())
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (hooks, file);
        Err("files are not available on the web".into())
    }
}

/// Load every xref of `d` that isn't unloaded; `host` is the drawing's own path.
pub(crate) fn load_all(d: &mut Drawing, host: Option<&str>) -> Vec<Outcome> {
    let names: Vec<String> = d
        .blocks
        .values()
        .filter(|b| b.xref_path.is_some() && b.xref_status != XrefStatus::Unloaded && !b.name.contains('|'))
        .map(|b| b.name.clone())
        .collect();
    names.iter().flat_map(|n| load(d, n, host)).collect()
}

/// Load (or reload) xref `name` of `d` from its file; `host` is the drawing's own path. Returns
/// the outcome, then notes about its nested xrefs.
pub(crate) fn load(d: &mut Drawing, name: &str, host: Option<&str>) -> Vec<Outcome> {
    let host_file = host.map(Path::new).filter(|p| p.file_name().is_some());
    let mut cx = Ctx { chain: host_file.map(canonical).into_iter().collect(), objects: 0, files: 0, notes: Vec::new() };
    let dir = folder(host);
    let (status, detail) = load_in(d, name, dir.as_deref(), &mut cx, 0);
    let key = d.block(name).map(|b| b.name.clone()).unwrap_or_else(|| name.to_string());
    let mut out = vec![Outcome { name: key, status, detail }];
    out.extend(cx.notes.into_iter().map(|n| Outcome { name: String::new(), status: XrefStatus::Unresolved, detail: n }));
    out
}

fn load_in(d: &mut Drawing, name: &str, dir: Option<&Path>, cx: &mut Ctx, depth: usize) -> (XrefStatus, String) {
    let Some(b) = d.block(name) else { return (XrefStatus::NotFound, "no such xref".into()) };
    let key = b.name.clone();
    let Some(path) = b.xref_path.clone() else { return (XrefStatus::Unresolved, "not an xref".into()) };
    clear(d, &key);
    let (status, detail) = match fetch(&path, dir, cx, depth) {
        Ok((x, file)) => match import(d, &key, &x, cx) {
            Ok(()) => (XrefStatus::Loaded, file.to_string_lossy().into_owned()),
            Err(e) => (XrefStatus::Unresolved, e),
        },
        Err(e) => e,
    };
    if status != XrefStatus::Loaded {
        // Whatever made it in before the failure goes again.
        clear(d, &key);
    }
    set_status(d, &key, status);
    (status, detail)
}

/// Read the drawing referenced as `path` and load its own attached xrefs into it.
fn fetch(path: &str, dir: Option<&Path>, cx: &mut Ctx, depth: usize) -> Result<(Drawing, PathBuf), (XrefStatus, String)> {
    let unresolved = |m: &str| (XrefStatus::Unresolved, m.to_string());
    if depth >= MAX_BLOCK_DEPTH {
        return Err(unresolved("xrefs are nested too deeply"));
    }
    let Some(file) = find(path, dir) else {
        let why = if cfg!(target_arch = "wasm32") { "files are not available on the web" } else { "file not found" };
        return Err((XrefStatus::NotFound, format!("{path}: {why}")));
    };
    let canon = canonical(&file);
    if cx.chain.contains(&canon) {
        return Err(unresolved("circular reference"));
    }
    if cx.files >= MAX_XREF_FILES {
        return Err(unresolved("too many referenced files"));
    }
    cx.files += 1;
    let mut x = read(&file).map_err(|e| (XrefStatus::Unresolved, format!("{}: {e}", file.to_string_lossy())))?;
    cx.chain.push(canon);
    let nested: Vec<String> =
        x.blocks.values().filter(|b| b.xref_path.is_some() && !b.xref_overlay && !b.name.contains('|')).map(|b| b.name.clone()).collect();
    let xdir = file.parent().map(Path::to_path_buf);
    for n in nested {
        let (status, detail) = load_in(&mut x, &n, xdir.as_deref(), cx, depth + 1);
        if status != XrefStatus::Loaded {
            cx.notes.push(format!("nested xref \"{n}\" not loaded: {detail}"));
        }
    }
    cx.chain.pop();
    Ok((x, file))
}

/// Bring referenced drawing `x` into xref `key`: its model space becomes the block's objects, its
/// symbols and blocks are added as `key|name` (dependent symbols the host already has keep the
/// host's settings, so layer changes survive a reload).
fn import(d: &mut Drawing, key: &str, x: &Drawing, cx: &mut Ctx) -> Result<(), String> {
    let count = x.blocks.values().fold(x.model.len(), |n, b| n.saturating_add(b.entities.len()));
    cx.objects = cx.objects.saturating_add(count);
    if cx.objects > MAX_XREF_OBJECTS {
        return Err(format!("more than {MAX_XREF_OBJECTS} objects"));
    }
    let dep = |sym: Sym, n: &str| (!shared(sym, n)).then(|| format!("{key}|{n}"));
    for l in &x.layers {
        if let Some(n) = dep(Sym::Layer, &l.name)
            && d.layer(&n).is_none()
        {
            let mut l = l.clone();
            l.name = n;
            l.linetype = dep(Sym::Linetype, &l.linetype).unwrap_or(l.linetype);
            d.layers.push(l);
        }
    }
    for t in &x.linetypes {
        if let Some(n) = dep(Sym::Linetype, &t.name)
            && d.linetype(&n).is_none()
        {
            d.linetypes.push(cadcraft_doc::Linetype { name: n, ..t.clone() });
        }
    }
    for t in &x.text_styles {
        if let Some(n) = dep(Sym::Style, &t.name)
            && d.text_style(&n).is_none()
        {
            d.text_styles.push(cadcraft_doc::TextStyle { name: n, ..t.clone() });
        }
    }
    for t in &x.dim_styles {
        if let Some(n) = dep(Sym::DimStyle, &t.name)
            && d.dim_style(&n).is_none()
        {
            let text_style = dep(Sym::Style, &t.text_style).unwrap_or_else(|| t.text_style.clone());
            d.dim_styles.push(cadcraft_doc::DimStyle { name: n, text_style, ..t.clone() });
        }
    }
    for t in &x.mleader_styles {
        if let Some(n) = dep(Sym::MLeaderStyle, &t.name)
            && !d.mleader_styles.iter().any(|o| o.name.eq_ignore_ascii_case(&n))
        {
            let text_style = dep(Sym::Style, &t.text_style).unwrap_or_else(|| t.text_style.clone());
            d.mleader_styles.push(cadcraft_doc::MLeaderStyle { name: n, text_style, ..t.clone() });
        }
    }
    for t in &x.table_styles {
        if let Some(n) = dep(Sym::TableStyle, &t.name)
            && !d.table_styles.iter().any(|o| o.name.eq_ignore_ascii_case(&n))
        {
            d.table_styles.push(cadcraft_doc::TableStyle { name: n, ..t.clone() });
        }
    }
    let adopt = |d: &mut Drawing, e: &Entity| {
        let mut e = e.clone();
        e.handle = d.new_handle();
        rename_refs(&mut e, &dep);
        if let EntityKind::Dimension(dm) = &mut e.kind {
            dm.assoc.clear();
        }
        e
    };
    for (name, b) in &x.blocks {
        let up = name.to_ascii_uppercase();
        if up.starts_with("*MODEL_SPACE") || up.starts_with("*PAPER_SPACE") {
            continue;
        }
        let mut nb = cadcraft_doc::Block::new(&format!("{key}|{name}"));
        nb.base = b.base;
        nb.description = b.description.clone();
        nb.anonymous = b.anonymous;
        nb.explodable = b.explodable;
        nb.units = b.units;
        for e in b.entities.iter() {
            let e = adopt(d, e);
            nb.entities.push(e);
        }
        d.blocks.insert(nb.name.clone(), Arc::new(nb));
    }
    let mut store = EntityStore::new();
    for e in x.model.iter() {
        let e = adopt(d, e);
        store.push(e);
    }
    let base = x.header.point("INSBASE").filter(|p| p.x.is_finite() && p.y.is_finite() && p.z.is_finite()).unwrap_or_default();
    let units = u8::try_from(x.header.i64("INSUNITS", 0)).unwrap_or(0);
    if let Some(b) = d.blocks.get_mut(key) {
        let b = Arc::make_mut(b);
        b.entities = store;
        b.base = base;
        b.units = units;
    }
    Ok(())
}

fn set_status(d: &mut Drawing, key: &str, status: XrefStatus) {
    if let Some(b) = d.blocks.get_mut(key) {
        Arc::make_mut(b).xref_status = status;
    }
}

/// Empty xref `key`: its objects and the blocks it brought go (its other symbols stay, with any
/// changes made to them, until it is detached).
pub(crate) fn clear(d: &mut Drawing, key: &str) {
    let prefix = format!("{key}|");
    d.blocks.retain(|n, _| !has_prefix(n, &prefix));
    if let Some(b) = d.blocks.get_mut(key)
        && !b.entities.is_empty()
    {
        Arc::make_mut(b).entities = EntityStore::new();
    }
}

/// Unload xref `key`: it stays attached but shows nothing until it is reloaded.
pub(crate) fn unload(d: &mut Drawing, key: &str) {
    clear(d, key);
    set_status(d, key, XrefStatus::Unloaded);
}

/// Detach xref `key`: its definition, every reference to it and every symbol it brought go.
/// Returns how many references were erased.
pub(crate) fn detach(d: &mut Drawing, key: &str) -> usize {
    clear(d, key);
    d.blocks.remove(key);
    let prefix = format!("{key}|");
    let mine = |n: &str| has_prefix(n, &prefix);
    d.layers.retain(|t| !mine(&t.name));
    d.linetypes.retain(|t| !mine(&t.name));
    d.text_styles.retain(|t| !mine(&t.name));
    d.dim_styles.retain(|t| !mine(&t.name));
    d.mleader_styles.retain(|t| !mine(&t.name));
    d.table_styles.retain(|t| !mine(&t.name));
    let refs = |st: &EntityStore| -> Vec<cadcraft_doc::Handle> {
        st.iter().filter(|e| matches!(&e.kind, EntityKind::Insert(i) if i.block.eq_ignore_ascii_case(key))).map(|e| e.handle).collect()
    };
    let mut hs = refs(&d.model);
    hs.extend(d.layouts.iter().flat_map(|l| refs(&l.entities)));
    let mut n = hs.len();
    for h in hs {
        d.remove_entity(h);
    }
    for b in d.blocks.values_mut() {
        let hs = refs(&b.entities);
        if !hs.is_empty() {
            n += hs.len();
            let b = Arc::make_mut(b);
            for h in hs {
                b.entities.remove(h);
            }
        }
    }
    n
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_resolve_against_the_host_folder_then_by_name() {
        let dir = Path::new("/proj/plans");
        let c = candidates("parts/a.dxf", Some(dir));
        assert_eq!(c, vec![dir.join("parts/a.dxf"), dir.join("a.dxf")]);
        let c = candidates("/abs/b.dwg", Some(dir));
        assert_eq!(c, vec![PathBuf::from("/abs/b.dwg"), dir.join("b.dwg")]);
        if !cfg!(windows) {
            // A Windows path saved elsewhere still finds the file next to the host.
            let c = candidates("C:\\jobs\\site.dwg", Some(dir));
            assert_eq!(c.last(), Some(&dir.join("site.dwg")));
        }
        assert_eq!(candidates("a.dxf", None), vec![PathBuf::from("a.dxf")]);
        assert!(find("", Some(dir)).is_none());
    }

    #[test]
    fn prefixes_ignore_case() {
        assert!(has_prefix("SITE|Walls", "site|"));
        assert!(!has_prefix("SITE", "site|"));
        assert!(!has_prefix("é", "ab"));
    }
}
