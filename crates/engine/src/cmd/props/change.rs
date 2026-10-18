//! Interactive CHPROP: selection, property name and value, repeated until Enter.
use crate::cmd::machines::{SelOutcome, SelectPhase};
use crate::{Accept, Input, Interactive, Prompt, Result, Session, Step};
use cadcraft_doc::Handle;
use serde_json::json;

#[derive(Default)]
pub(super) struct ChangeProps {
    selection: SelectPhase,
    handles: Vec<Handle>,
    property: Option<String>,
}
impl Interactive for ChangeProps {
    fn name(&self) -> &'static str {
        "CHPROP"
    }
    fn begin(&mut self, s: &mut Session) -> Result<Step> {
        self.selection = SelectPhase::begin(s);
        if self.selection.done {
            self.handles.clone_from(&self.selection.picked);
        }
        Ok(Step::Continue)
    }
    fn prompt(&self, _: &Session) -> Prompt {
        if self.handles.is_empty() {
            return self.selection.prompt();
        }
        if self.property.is_some() {
            return Prompt::new("Enter new property value", Accept::TEXT);
        }
        Prompt::new("Property [Color/Layer/Linetype/Lineweight/Ltscale/Transparency]", Accept::TEXT).kw(&[
            "Color",
            "Layer",
            "Linetype",
            "Lineweight",
            "Ltscale",
            "Transparency",
        ])
    }
    fn input(&mut self, s: &mut Session, input: Input) -> Result<Step> {
        if self.handles.is_empty() {
            return match self.selection.feed(s, &input)? {
                SelOutcome::More => Ok(Step::Continue),
                SelOutcome::Empty => Ok(Step::Done),
                SelOutcome::Done(hs) => {
                    self.handles = hs;
                    Ok(Step::Continue)
                }
            };
        }
        let text = match input {
            Input::Enter => return Ok(Step::Done),
            Input::Text(t) | Input::Keyword(t) => t,
            _ => return Ok(Step::Continue),
        };
        if let Some(property) = &self.property {
            let mut params = json!({"handles": self.handles.iter().map(|h| h.hex()).collect::<Vec<_>>()});
            params[property] = match property.as_str() {
                "lineweight" | "ltscale" | "transparency" => {
                    text.parse::<f64>().ok().filter(|v| v.is_finite()).map_or_else(|| json!(text), |v| json!(v))
                }
                _ => json!(text),
            };
            super::run_set(s, &params)?;
            self.property = None;
        } else {
            let p = text.trim().to_ascii_lowercase();
            if !["color", "layer", "linetype", "lineweight", "ltscale", "transparency"].contains(&p.as_str()) {
                return Err(crate::cmd::bad("chprop", "unknown property"));
            }
            self.property = Some(p);
        }
        Ok(Step::Continue)
    }
}
