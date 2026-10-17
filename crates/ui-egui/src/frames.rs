//! Title-block frame detection over model space: closed frame-sized polylines become plot
//! windows; the texts inside each frame supply the sheet fields (图名/图号/版别) for the sheet
//! index and batch plot. Heuristics, not CAD-precise: real title blocks vary endlessly, the
//! dialogs let the user untick or re-run.

use cadcraft_doc::{Drawing, EntityKind, Space};
use cadcraft_geom::Bounds2;

/// One detected drawing frame and the sheet fields read from the texts inside it.
#[derive(Clone, Debug)]
pub struct Frame {
    /// The frame's bounds in model units — the batch-plot window.
    pub window: Bounds2,
    /// 图名: the longest text inside the frame.
    pub name: String,
    /// 图号: the text next to a "图号"-style label, or a short code-like text.
    pub number: String,
    /// 版别: the text next to a revision label, or empty when none was found.
    pub revision: String,
}

// Frame sizes between A3 landscape and 2×A0, with slack: title blocks are drawn as rectangles
// near standard sheet sizes; details, schedules and hatches never reach these magnitudes.
const MIN_W: f64 = 12_000.0;
const MIN_H: f64 = 7_000.0;
const MAX_W: f64 = 95_000.0;
const MAX_H: f64 = 70_000.0;

fn frame_sized(b: &Bounds2) -> bool {
    let (w, h) = (b.max.x - b.min.x, b.max.y - b.min.y);
    (MIN_W..=MAX_W).contains(&w) && (MIN_H..=MAX_H).contains(&h)
}

fn contains(b: &Bounds2, x: f64, y: f64, margin: f64) -> bool {
    x >= b.min.x - margin && x <= b.max.x + margin && y >= b.min.y - margin && y <= b.max.y + margin
}

/// A text with its insertion point, height and content.
struct Txt {
    x: f64,
    y: f64,
    h: f64,
    s: String,
}

impl Txt {
    fn chars(&self) -> usize {
        self.s.chars().count()
    }
}

/// Detect frames in model space, in reading order (top band first, then left to right).
pub fn detect(d: &Drawing) -> Vec<Frame> {
    let Some(store) = d.space(&Space::Model) else {
        return Vec::new();
    };
    let mut windows: Vec<Bounds2> = Vec::new();
    let mut texts: Vec<Txt> = Vec::new();
    for e in store.iter() {
        match &e.kind {
            EntityKind::LwPolyline(pl) if pl.closed && pl.vertices.len() >= 3 => {
                let b = Bounds2::from_points(pl.vertices.iter().map(|v| v.p));
                if frame_sized(&b) {
                    windows.push(b);
                }
            }
            EntityKind::Text(t) => texts.push(Txt { x: t.insert.x, y: t.insert.y, h: t.height, s: t.value.clone() }),
            EntityKind::MText(m) => texts.push(Txt { x: m.insert.x, y: m.insert.y, h: m.height, s: m.contents.clone() }),
            _ => {}
        }
    }
    // A frame drawn inside another (a detail inside an A0) would plot twice; keep the outer one.
    let outer: Vec<Bounds2> = windows
        .iter()
        .filter(|b| !windows.iter().any(|o| o != *b && contains(o, b.min.x, b.min.y, 1.0) && contains(o, b.max.x, b.max.y, 1.0)))
        .copied()
        .collect();
    let windows = outer;

    let mut frames: Vec<Frame> = windows
        .into_iter()
        .map(|b| {
            let inside: Vec<&Txt> = texts.iter().filter(|t| contains(&b, t.x, t.y, 0.0)).collect();
            // 图名 lives in the title strip at the frame's foot: the tallest text there beats the
            // longer subtitle/schedule lines elsewhere in the sheet.
            let strip_top = b.min.y + (b.max.y - b.min.y) * 0.15;
            let strip: Vec<&Txt> = inside.iter().copied().filter(|t| t.y <= strip_top).collect();
            let pool = if strip.is_empty() { &inside } else { &strip };
            let name = pool.iter().max_by_key(|t| (t.h * 1000.0) as i64).map(|t| t.s.clone()).unwrap_or_default();
            let number = field_by_label(&inside, &["图号", "圖號", " drawing no", "dwg no"]).unwrap_or_else(|| {
                // A short code-looking text: letters/digits with a separator, unlike the long name.
                inside
                    .iter()
                    .map(|t| t.s.trim())
                    .filter(|s| s.len() <= 20 && s.contains('-') && s.chars().any(|c| c.is_ascii_digit()) && *s != name)
                    .next_back()
                    .unwrap_or_default()
                    .to_string()
            });
            let revision = field_by_label(&inside, &["版别", "版本", "版次", "圖版", "rev", "Rev"]).unwrap_or_default();
            Frame { window: b, name, number, revision }
        })
        .collect();
    frames.sort_by(|a, b| {
        // Reading order: upper bands first, then left to right inside a band.
        let band = |f: &Frame| (f.window.max.y / 10_000.0).round();
        band(b).total_cmp(&band(a)).then_with(|| a.window.min.x.total_cmp(&b.window.min.x))
    });
    frames
}

/// The text nearest below/right of a label like "图号"; sheet fields sit beside their labels.
fn field_by_label<'a>(inside: &[&'a Txt], labels: &[&str]) -> Option<String> {
    let label = inside.iter().find(|t| labels.iter().any(|l| t.s.to_lowercase().contains(&l.to_lowercase())))?;
    let candidates = inside
        .iter()
        .filter(|t| !std::ptr::eq(**t, *label) && t.s.trim() != label.s.trim())
        .filter(|t| {
            // Same line to the right, or the nearest line below the label.
            (t.y - label.y).abs() <= label.h * 2.0 && t.x > label.x || t.y < label.y && (t.y - label.y).abs() <= label.h * 6.0
        });
    candidates.min_by(|a, b| {
        let da = (a.x - label.x).powi(2) + (a.y - label.y).powi(2);
        let db = (b.x - label.x).powi(2) + (b.y - label.y).powi(2);
        da.total_cmp(&db)
    })
    .map(|t| t.s.clone())
}
