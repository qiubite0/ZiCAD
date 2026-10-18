use cadcraft_engine::Session;
use cadcraft_engine::doc::color::Color;
use serde_json::json;

#[test]
fn change_properties_and_reset_by_layer_are_undoable() {
    let mut s = Session::new();
    s.execute("line", &json!({"points": [[0,0],[10,0]]})).unwrap();
    let h = s.doc().unwrap().model.last().unwrap().handle;
    s.execute("chprop", &json!({"handles":[h.hex()],"color":"red","ltscale":2})).unwrap();
    assert_eq!(s.doc().unwrap().entity(h).unwrap().common.color, Color::Index(1));
    s.set_selection(vec![h]);
    s.execute("setbylayer", &json!({})).unwrap();
    let e = s.doc().unwrap().entity(h).unwrap();
    assert_eq!(e.common.color, Color::ByLayer);
    assert_eq!(e.common.ltscale, 2.0);
    s.execute("undo", &json!({})).unwrap();
    assert_eq!(s.doc().unwrap().entity(h).unwrap().common.color, Color::Index(1));
}

#[test]
fn chprop_interactive_selection_and_multiple_properties() {
    use cadcraft_engine::Input;
    let mut s = Session::new();
    s.execute("line", &json!({"points": [[0,0],[10,0]]})).unwrap();
    let h = s.doc().unwrap().model.last().unwrap().handle;
    s.cmdline("chprop").unwrap();
    s.input(Input::Pick(vec![h])).unwrap();
    s.input(Input::Enter).unwrap();
    s.input(Input::Keyword("Color".into())).unwrap();
    s.input(Input::Text("red".into())).unwrap();
    s.input(Input::Keyword("Ltscale".into())).unwrap();
    s.input(Input::Text("2.5".into())).unwrap();
    s.input(Input::Enter).unwrap();
    let e = s.doc().unwrap().entity(h).unwrap();
    assert_eq!(e.common.color, Color::Index(1));
    assert_eq!(e.common.ltscale, 2.5);
    s.execute("undo", &json!({})).unwrap();
    assert_eq!(s.doc().unwrap().entity(h).unwrap().common.color, Color::ByLayer);
}
