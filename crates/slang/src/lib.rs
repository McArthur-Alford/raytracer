use std::collections::HashMap;

use crate::{representation::Reflection, traversal::walk_json};

mod backend;
mod key;
mod representation;
mod slang_program;
mod traversal;
// mod webgpu;

pub use traversal::Position;

pub fn generate_hashmap(json: &str) -> serde_json::Result<HashMap<String, Position>> {
    let reflection: Reflection = serde_json::from_str(json)?;
    let mut out = HashMap::new();
    walk_json(reflection, &mut out);
    Ok(out)
}
