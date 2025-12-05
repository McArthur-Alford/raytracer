use std::{collections::HashMap, ops::Add};

use crate::representation::{Binding, Parameter, Reflection, Type};

#[derive(Debug, Default, Clone, Copy)]
pub struct Position {
    pub bind_group: usize,
    pub bind_index: usize,
    pub byte_offset: usize,
}

pub fn walk_json(reflection: Reflection, out: &mut HashMap<String, Position>) {
    let position = Position::default();
    for param in reflection.parameters {
        let position = Position::default();
        walk_param("params", param, position, out);
    }
    for ep in reflection.entry_points {
        let position = position.clone();
        for param in ep.parameters {
            walk_param(&ep.name, param, position, out);
        }
    }
    dbg!(out);
}

pub fn walk_param(
    prefix: &str,
    param: Parameter,
    mut position: Position,
    out: &mut HashMap<String, Position>,
) {
    for binding in param.bindings.collect() {
        match Some(binding) {
            Some(Binding::VaryingInput { index, count }) => {}
            Some(Binding::VaryingOutput { index, count }) => {}
            Some(Binding::DescriptorTableSlot { index, count }) => {
                position.bind_index += index as usize;
            }
            Some(Binding::SubElementRegisterSpace { index, count }) => {
                position.bind_group += index as usize;
                position.bind_group = position.bind_group.max(1);
            }
            Some(Binding::Uniform {
                offset,
                size,
                element_stride,
            }) => {
                dbg!(size);
                dbg!(element_stride);
                position.byte_offset += offset as usize;
            }
            None => {}
        }
    }

    let name = param.name.clone();

    if let Some(ty) = param.ty {
        walk_type(prefix, ty, position, out, name);
    }
}

fn walk_type(
    prefix: &str,
    ty: Type,
    position: Position,
    out: &mut HashMap<String, Position>,
    name: String,
) {
    match ty {
        Type::Struct {
            name: type_name,
            fields,
        } => {
            out.insert(format!("{}.{}", prefix, name), position);
            for field in fields {
                walk_param(&format!("{}.{}", prefix, name), field, position, out);
            }
        }
        Type::Resource { .. } | Type::SamplerState {} => {
            out.insert(format!("{}.{}", prefix, name), position);
        }
        Type::Array {
            element_count,
            element_type,
            uniform_stride,
        } => walk_type(prefix, *element_type, position, out, name),
        Type::ParameterBlock {
            element_type,
            container_var_layout,
            element_var_layout,
        } => walk_type(prefix, *element_type, position, out, name),
        _ => {
            out.insert(format!("{}.{}", prefix, name), position);
        }
    }
}
