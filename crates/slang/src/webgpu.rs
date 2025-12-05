// use std::collections::HashMap;

// use wgpu::BindGroupLayout;

// use crate::{Position, key::Key};

// type Binding<'a> = (wgpu::BindGroupLayoutEntry, wgpu::BindGroupEntry<'a>);

// pub enum WebGPUResource {
//     Texture(wgpu::Texture),
// }

// struct WebGPUResourceNode {
//     /// The resource to be passed into a bind.
//     /// Handle is cloned so make sure it doesn't
//     /// become out of date please.
//     resource: WebGPUResource,
//     /// All binds (by key) that consume this resource
//     /// and need to be re-bound when updated.
//     downstream: Vec<String>,
// }

// struct WebGPUPipeline {
//     name: String,
//     fields: Vec<String>,
// }

// struct WebGPUEntryPoint {}

// struct WebGPUProgram {
//     positions: HashMap<String, Position>,
//     entry_points: HashMap<String, WebGPUEntryPoint>,
// }

// pub struct WebGPUContext {
//     programs: HashMap<String, WebGPUProgram>,
// }

// impl WebGPUContext {
//     /// Slots in a resource at a particular field.
//     /// If the resource was already present at that field,
//     /// triggers a rebind of all downstream entries (if they exist).
//     pub fn resource(&mut self, key: impl Key, resource: WebGPUResource) {}

//     /// Slots in a bind generation function at a particular field.
//     /// This function produces the BindGroupEntry and BindGroupLayoutEntry.
//     pub fn bind<'a, F>(&mut self, key: impl Key, f: F)
//     where
//         F: Fn(usize, &'a WebGPUResource) -> Binding<'a>,
//     {
//     }

//     /// Creates or updates the fields associated with a pipeline of
//     /// the given name. This is the best way i can think to pull out
//     /// all the groups given we dont know how many/what they are.
//     pub fn pipeline(&mut self, pipeline: String, keys: impl IntoIterator<Item = impl Key>) {}

//     /// Gets all bind group layouts (in order) of a given pipeline
//     /// and their bind group index (the usize).
//     pub fn get_bind_group_layouts(
//         &mut self,
//         pipeline: String,
//     ) -> Vec<(usize, wgpu::BindGroupLayout)> {
//         todo!()
//     }

//     /// Gets all bind groups (in order) of a given pipeline
//     /// and their bind group index (the usize).
//     pub fn get_bind_groups(&mut self, pipeline: String) -> Vec<(usize, wgpu::BindGroup)> {
//         todo!()
//     }

//     /// Gets the group specified by usize that belongs to a
//     /// particular function + entrypoint.
//     pub fn get_group(&mut self, group: usize) -> wgpu::BindGroup {
//         todo!()
//     }
// }
