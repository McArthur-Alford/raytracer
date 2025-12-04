use crate::{backend::Backend, key::Key, slang_program::SlangProgram};

pub struct WebGPU;

// We need a layout entry and resource
// The rest of bindgroup stuff can be auto-generated
// Not sure how much of a fan of this i am? but its better
// than rewriting all the descriptors without bindings lol.
//
// TODO: Maybe explore a closure approach that generates
// them given a binding input?
#[derive(Default)]
pub struct WebGPUAssociation<'a> {
    bg_layout_entry: Option<wgpu::BindGroupLayoutEntry>,
    resource: Option<wgpu::BindingResource<'a>>,
}

impl Backend for WebGPU {
    type Association<'a> = WebGPUAssociation<'a>;
}

impl<'a> SlangProgram<'a, WebGPU> {
    // bg_layout_entry: Option<wgpu::BindGroupLayoutEntry>,
    // resource: Option<wgpu::BindingResource<'a>>,
    pub fn register_bg_layout_entry(
        &mut self,
        field: impl Key,
        bg_layout_entry: wgpu::BindGroupLayoutEntry,
    ) -> Result<(), String> {
        let field = *self
            .fields
            .get(&field.build())
            .ok_or(Err("MAGIC".to_owned()))?;
        Ok(())
    }
}
