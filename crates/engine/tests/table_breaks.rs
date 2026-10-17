//! Table breaking (#534): parts with repeated labels, part grips, TABLEDIT in any part; data
//! links filling a table from a CSV file and DATALINKUPDATE.

use cadcraft_engine::Session;
use cadcraft_engine::doc::{BreakDirection, EntityKind, Handle, Table};
use cadcraft_engine::geom::Vec2;
use cadcraft_engine::render;
use serde_json::json;

fn table(s: &Session, h: &str) -> Table {
    match &s.doc().unwrap().entity(Handle::parse_hex(h).unwrap()).unwrap().kind {
        EntityKind::Table(t) => t.clone(),
        _ => panic!("not a table"),
    }
}

/// Title, header and 8 data rows, each 1 high, 2 columns of 3.
fn session() -> (Session, String) {
    let mut s = Session::new();
    let cells: Vec<Vec<String>> = (1..=8).map(|i| vec![format!("r{i}"), i.to_string()]).collect();
    let r = s
        .execute("table", &json!({ "at": [0, 0], "title": "T", "header": ["Name", "N"], "cells": cells, "rowHeight": 1.0, "colWidth": 3.0 }))
        .unwrap();
    let h = r["handle"].as_str().unwrap().to_string();
    // Same height for the title row.
    let hh = Handle::parse_hex(&h).unwrap();
    s.doc_mut()
        .unwrap()
        .modify_entity(hh, |e| {
            if let EntityKind::Table(t) = &mut e.kind {
                t.row_heights[0] = 1.0;
            }
        })
        .unwrap();
    (s, h)
}

#[test]
fn breaking_draws_parts_with_repeated_labels() {
    let (mut s, h) = session();
    let r = s.execute("table.break", &json!({ "handle": h, "height": 6.0, "spacing": 1.0 })).unwrap();
    // 2 label rows + 4 data rows per part.
    assert_eq!(r["parts"], 2);
    let t = table(&s, &h);
    let g = render::table_geometry(s.doc().unwrap(), &t);
    // The header text appears in both parts.
    assert_eq!(g.texts.iter().filter(|x| x.text == "Name").count(), 2);
    let r5 = g.texts.iter().find(|x| x.text == "r5").unwrap();
    assert_eq!(r5.row, 6);
    assert!(r5.at.x > 7.0 && r5.at.y > -3.0, "r5 starts the second part at x 7: {:?}", r5.at);
    // Extents cover both parts.
    let b = cadcraft_engine::doc::entity_bounds(s.doc().unwrap(), s.doc().unwrap().entity(Handle::parse_hex(&h).unwrap()).unwrap(), 0);
    assert!((b.max.x - 13.0).abs() < 1e-9 && (b.min.y + 6.0).abs() < 1e-9, "{b:?}");
    // TABLEDIT finds cells in the second part.
    s.execute("tabledit", &json!({ "at": [11.5, -2.5], "text": "five" })).unwrap();
    assert_eq!(table(&s, &h).cells[6][1].text, "five");
    // Down, with manual heights; Off.
    s.execute("table.break", &json!({ "handle": h, "direction": "down", "heights": [4.0] })).unwrap();
    let t = table(&s, &h);
    assert_eq!(t.breaks.as_ref().unwrap().direction, BreakDirection::Down);
    assert_eq!(t.parts().first().unwrap().rows, vec![0, 1, 2, 3]);
    let r = s.execute("table.break", &json!({ "handle": h, "enabled": false })).unwrap();
    assert_eq!(r["parts"], 1);
    assert!(s.execute("table.break", &json!({ "handle": h, "direction": "up" })).is_err());
    assert!(s.execute("table.break", &json!({ "handle": h, "height": -1 })).is_err());
}

#[test]
fn part_grips_move_parts() {
    let (mut s, h) = session();
    s.execute("table.break", &json!({ "handle": h, "height": 6.0, "spacing": 1.0 })).unwrap();
    let k = EntityKind::Table(table(&s, &h));
    let grips = k.grips();
    assert_eq!(grips, vec![Vec2::new(0.0, 0.0), Vec2::new(7.0, 0.0)]);
    // Drag the second part's grip; the first part stays.
    let moved = cadcraft_engine::grips::stretch_grip(&k, 1, Vec2::new(9.0, -2.0)).unwrap();
    let EntityKind::Table(t) = &moved else { panic!() };
    let parts = t.parts();
    assert_eq!((parts[0].origin, parts[1].origin), (Vec2::new(0.0, 0.0), Vec2::new(9.0, -2.0)));
    // The insertion grip moves everything.
    let all = cadcraft_engine::grips::stretch_grip(&moved, 0, Vec2::new(1.0, 1.0)).unwrap();
    let EntityKind::Table(t) = &all else { panic!() };
    assert_eq!(t.parts()[1].origin, Vec2::new(10.0, -1.0));
    // Typed: select the table by a point, then the height.
    s.execute("table.break", &json!({ "handle": h, "enabled": false })).unwrap();
    s.cmdline("table.break").unwrap();
    s.cmdline("1,-0.5").unwrap();
    s.cmdline("4").unwrap();
    assert_eq!(table(&s, &h).parts().len(), 4);
}

#[test]
fn data_links_fill_and_update() {
    let dir = std::env::temp_dir().join(format!("cadcraft-datalink-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("parts.csv");
    std::fs::write(&path, "Part,Qty,Note\r\nbolt,4,\"M6, zinc\"\nnut,8,\n").unwrap();
    let mut s = Session::new();
    let r = s.execute("table", &json!({ "at": [0, 0], "title": "Parts", "cols": 2, "rows": 1 })).unwrap();
    let h = r["handle"].as_str().unwrap().to_string();
    let r = s.execute("datalink", &json!({ "handle": h, "path": path.to_str().unwrap() })).unwrap();
    assert_eq!((r["rows"].as_u64(), r["cols"].as_u64()), (Some(4), Some(3)));
    let t = table(&s, &h);
    assert_eq!(t.cells[0][0].merged, Some((1, 3)));
    assert_eq!(t.cells[1][0].text, "Part");
    assert_eq!(t.cells[2][2].text, "M6, zinc");
    // An update re-reads the file and resizes the table to it.
    std::fs::write(&path, "Part,Qty\nwasher,12\n").unwrap();
    let r = s.execute("datalinkupdate", &json!({})).unwrap();
    assert_eq!(r["updated"], 1);
    let t = table(&s, &h);
    assert_eq!((t.cells.len(), t.col_widths.len()), (3, 2));
    assert_eq!(t.cells[2][1].text, "12");
    // Typed DATALINKUPDATE with a selection.
    s.cmdline("datalinkupdate").unwrap();
    s.cmdline("1,-0.2").unwrap();
    s.cmdline("").unwrap();
    // Missing files and missing paths are errors, not panics.
    assert!(s.execute("datalink", &json!({ "handle": h })).is_err());
    assert!(s.execute("datalink", &json!({ "handle": h, "path": dir.join("missing.csv").to_str().unwrap() })).is_err());
    std::fs::remove_file(&path).unwrap();
    let r = s.execute("datalinkupdate", &json!({})).unwrap();
    assert_eq!(r["updated"], 0);
    s.execute("datalink", &json!({ "handle": h, "remove": true })).unwrap();
    assert!(table(&s, &h).data_link.is_none());
    let _ = std::fs::remove_dir_all(&dir);
}
