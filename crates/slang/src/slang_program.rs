use std::{
    collections::{HashMap, HashSet},
    io::Read,
    path::Path,
};

use crate::{Position, backend::Backend, generate_hashmap, key::Key};

// TODO if lifetimes/references become an issue
// allow SlangProgram to spit out a pipelineBuilder
// which is more short lived, has a ref to the superior slang program,
// and has a final .build() method which consumes it?

#[derive(Debug)]
pub enum Error {
    UknownField,
    MissingGroup,
    MissingBind,
    MissingGroupFunction,
    Io(std::io::Error),
    SerdeJson(serde_json::Error),
}

struct BindEntry<'a, T1> {
    /// T1 data for this bind entry
    data: Option<T1>,
    /// Callback to generate T1 given a bind index:
    func: Box<dyn Fn(usize) -> T1 + 'a>,
}

struct BindGroupEntry<'a, T1, T2> {
    /// Bind group data, T2
    data: Option<T2>,
    /// Maps bind index -> Bind data T1
    binds: HashMap<usize, BindEntry<'a, T1>>,
}

impl<'a, T1, T2> Default for BindGroupEntry<'a, T1, T2> {
    fn default() -> Self {
        Self {
            data: Default::default(),
            binds: Default::default(),
        }
    }
}

pub struct SlangProgram {
    // Maps field key -> position
    positions: HashMap<String, Position>,
}

impl SlangProgram {
    pub fn new(json: &str) -> Result<Self, Error> {
        // let path = reflection_path.as_ref();
        // let out = std::fs::read_to_string(path).map_err(|e| Error::Io(e))?;
        let reflect = generate_hashmap(&json).map_err(|e| Error::SerdeJson(e))?;

        Ok(Self { positions: reflect })
    }

    pub fn new_pipeline<T1, T2>(&self) -> PipelineBuilder<T1, T2> {
        PipelineBuilder {
            program: &self,
            groups: HashMap::new(),
            func: None,
        }
    }
}

pub struct PipelineBuilder<'a, 'b, T1, T2> {
    program: &'b SlangProgram,
    // Maps bind group -> BindGroupEntry
    groups: HashMap<usize, BindGroupEntry<'a, T1, T2>>,
    /// Callback to generate T2 given a bind group and entries
    /// ordered by bind group:
    func: Option<Box<dyn Fn(usize, Vec<(usize, T1)>) -> T2 + 'a>>,
}

impl<'a, T1, T2> BindGroupEntry<'a, T1, T2> {
    fn build(&mut self) {
        for (k, v) in &mut self.binds {
            v.data = Some((v.func)(*k));
        }
    }
}

impl<'a, 'b, T1, T2> PipelineBuilder<'a, 'b, T1, T2> {
    pub fn pipeline<F>() {}

    pub fn group<F>(&mut self, f: F) -> &mut Self
    where
        F: Fn(usize, Vec<(usize, T1)>) -> T2 + 'a,
    {
        self.func = Some(Box::new(f));
        self
    }

    pub fn entry<F>(&mut self, key: &impl Key, f: F) -> Result<&mut Self, Error>
    where
        F: (Fn(usize) -> T1) + 'a,
    {
        let pos = self
            .program
            .positions
            .get(&key.build())
            .ok_or(Error::UknownField)?;

        let entry = self
            .groups
            .entry(pos.bind_group)
            .or_insert(BindGroupEntry::default());

        entry.binds.insert(
            pos.bind_index,
            BindEntry {
                data: None,
                func: Box::new(f) as Box<dyn (Fn(usize) -> T1) + 'a>,
            },
        );

        Ok(self)
    }

    pub fn build(&mut self) -> Result<Vec<T2>, Error> {
        let groups: HashSet<usize> = self
            .program
            .positions
            .values()
            .map(|p| p.bind_group)
            .collect();

        let mut out = Vec::new();
        let f = self.func.as_ref().ok_or(Error::MissingGroupFunction)?;

        // For each bind group
        for gid in groups {
            // Get the group to intialize its bind data
            let mut group = self.groups.remove(&gid).ok_or(Error::MissingGroup)?;

            group.build(); // all t1s in group are populated

            out.push((f)(
                gid,
                group
                    .binds
                    .into_iter()
                    .filter_map(|(i, b)| Some((i, b.data?)))
                    .collect(),
            ));
        }

        Ok(out)
    }
}
