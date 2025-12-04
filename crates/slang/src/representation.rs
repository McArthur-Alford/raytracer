use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Reflection {
    pub parameters: Vec<Parameter>,
    #[serde(rename = "entryPoints")]
    pub entry_points: Vec<EntryPoint>,
}

#[derive(Debug, Deserialize)]
pub struct EntryPoint {
    pub name: String,
    pub stage: Stage,
    pub parameters: Vec<Parameter>,
    pub result: Parameter,
    pub bindings: Vec<Binding>,
}

#[derive(Debug, Deserialize, Default)]
pub struct BindingsField {
    #[serde(default)]
    pub binding: Option<Binding>,
    #[serde(default)]
    pub bindings: Vec<Binding>,
}

impl BindingsField {
    fn collect(self) -> Vec<Binding> {
        let mut v = self.bindings;
        if let Some(b) = self.binding {
            v.push(b);
        }
        v
    }
}

#[derive(Debug, Deserialize)]
pub struct Parameter {
    #[serde(default)]
    pub name: String,

    #[serde(default)]
    pub stage: Option<Stage>,

    #[serde(default, rename = "semanticName")]
    pub semantic_name: Option<String>,

    #[serde(flatten)]
    pub bindings: BindingsField,

    #[serde(default, rename = "type")]
    pub ty: Option<Type>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Stage {
    Vertex,
    Fragment,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Binding {
    VaryingInput {
        index: u32,
        #[serde(default)]
        count: Option<u32>,
    },
    VaryingOutput {
        index: u32,
        #[serde(default)]
        count: Option<u32>,
    },
    DescriptorTableSlot {
        index: u32,
        #[serde(default)]
        count: Option<u32>,
    },
    SubElementRegisterSpace {
        index: u32,
        #[serde(default)]
        count: Option<u32>,
    },
    Uniform {
        offset: u32,
        size: u32,
        #[serde(default, rename = "elementStride")]
        element_stride: Option<u32>,
    },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Type {
    Struct {
        name: String,
        fields: Vec<Parameter>,
    },

    Vector {
        #[serde(rename = "elementCount")]
        element_count: u32,
        #[serde(rename = "elementType")]
        element_type: Box<Type>,
    },

    Scalar {
        #[serde(rename = "scalarType")]
        scalar_type: ScalarType,
    },

    Resource {
        #[serde(rename = "baseShape")]
        base_shape: BaseShape,
        #[serde(rename = "resultType")]
        result_type: Box<Type>,
    },

    SamplerState {},

    Matrix {
        #[serde(rename = "rowCount")]
        row_count: u32,
        #[serde(rename = "columnCount")]
        column_count: u32,
        #[serde(rename = "elementType")]
        element_type: Box<Type>,
    },

    Array {
        #[serde(rename = "elementCount")]
        element_count: u32,
        #[serde(rename = "elementType")]
        element_type: Box<Type>,
        #[serde(default, rename = "uniformStride")]
        uniform_stride: Option<u32>,
    },

    ParameterBlock {
        #[serde(rename = "elementType")]
        element_type: Box<Type>,

        #[serde(default, rename = "containerVarLayout")]
        container_var_layout: Option<Box<Parameter>>,

        #[serde(default, rename = "elementVarLayout")]
        element_var_layout: Option<Box<Parameter>>,
    },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ScalarType {
    Float32,
    Float64,
    Int32,
    Uint32,
    Bool,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BaseShape {
    Texture2D,
    TextureCube,
    #[serde(other)]
    Unknown,
}
