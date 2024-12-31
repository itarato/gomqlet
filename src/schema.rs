use std::fs::File;

use serde_json::Value;

#[derive(Debug)]
pub enum TypeClass {
    NonNull(Box<TypeClass>),
    List(Box<TypeClass>),
    Object(String),
    Enum(String),
    Interface(String),
    Scalar(String),
    Input(String),
    Union(String),
}

impl TypeClass {
    fn underlying_type_name(&self) -> Option<String> {
        match self {
            TypeClass::Object(name) => Some(name.clone()),
            TypeClass::Enum(name) => Some(name.clone()),
            TypeClass::Interface(name) => Some(name.clone()),
            TypeClass::Scalar(name) => Some(name.clone()),
            TypeClass::Input(name) => Some(name.clone()),
            TypeClass::Union(name) => Some(name.clone()),
            TypeClass::NonNull(inner) => inner.underlying_type_name(),
            TypeClass::List(inner) => inner.underlying_type_name(),
        }
    }

    fn from_json_value(node: &Value) -> TypeClass {
        let kind = node.as_object().unwrap()["kind"].as_str().unwrap();
        match kind {
            "NON_NULL" => TypeClass::NonNull(Box::new(TypeClass::from_json_value(
                &node.as_object().unwrap()["ofType"],
            ))),
            "LIST" => TypeClass::List(Box::new(TypeClass::from_json_value(
                &node.as_object().unwrap()["ofType"],
            ))),
            "OBJECT" => TypeClass::Object(
                node.as_object().unwrap()["name"]
                    .as_str()
                    .unwrap()
                    .to_string(),
            ),
            "INTERFACE" => TypeClass::Interface(
                node.as_object().unwrap()["name"]
                    .as_str()
                    .unwrap()
                    .to_string(),
            ),
            "SCALAR" => TypeClass::Scalar(
                node.as_object().unwrap()["name"]
                    .as_str()
                    .unwrap()
                    .to_string(),
            ),
            "INPUT_OBJECT" => TypeClass::Input(
                node.as_object().unwrap()["name"]
                    .as_str()
                    .unwrap()
                    .to_string(),
            ),
            "ENUM" => TypeClass::Enum(
                node.as_object().unwrap()["name"]
                    .as_str()
                    .unwrap()
                    .to_string(),
            ),
            "UNION" => TypeClass::Union(
                node.as_object().unwrap()["name"]
                    .as_str()
                    .unwrap()
                    .to_string(),
            ),
            _ => unimplemented!("Unmapped field type: {}", kind),
        }
    }
}

pub struct Arg {
    pub name: String,
    pub arg_type: TypeClass,
}

impl Arg {
    fn from_json_value(node: &Value) -> Arg {
        let object = node.as_object().unwrap();
        let name = object["name"].as_str().unwrap().to_string();
        let arg_type = TypeClass::from_json_value(&object["type"]);

        Arg { name, arg_type }
    }
}

pub struct ArgList {
    elems: Vec<Arg>,
}

impl ArgList {
    pub fn arg_names(&self, prefix: &String) -> Vec<String> {
        self.elems
            .iter()
            .filter_map(|arg| {
                if arg.name.starts_with(prefix) {
                    Some(arg.name.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn arg(&self, name: &String) -> Option<&Arg> {
        for arg in &self.elems {
            if &arg.name == name {
                return Some(arg);
            }
        }

        None
    }
}

pub struct Field {
    pub name: String,
    field_type: TypeClass,
    pub args: ArgList,
}

impl Field {
    fn from_json_value(node: &Value) -> Field {
        let name = node.as_object().unwrap()["name"]
            .as_str()
            .unwrap()
            .to_string();

        Field {
            name,
            field_type: TypeClass::from_json_value(&node.as_object().unwrap()["type"]),
            args: ArgList {
                elems: Field::resolve_args(node.as_object().unwrap()["args"].as_array().unwrap()),
            },
        }
    }

    fn resolve_args(raw_args: &Vec<Value>) -> Vec<Arg> {
        raw_args
            .iter()
            .map(|raw_arg| Arg {
                name: raw_arg.as_object().unwrap()["name"]
                    .as_str()
                    .unwrap()
                    .to_string(),
                arg_type: TypeClass::from_json_value(&raw_arg.as_object().unwrap()["type"]),
            })
            .collect()
    }
}

pub struct ObjectType {
    name: String,
    fields: Vec<Field>,
}

pub struct InputObjectType {
    pub name: String,
    pub args: ArgList,
}

pub enum Type {
    Object(ObjectType),
    InputObject(InputObjectType),
}

impl Type {
    pub fn from_json_value(node: &Value) -> Option<Type> {
        let object = node.as_object().unwrap();
        let name = object["name"].as_str().unwrap().to_string();
        let kind = object["kind"].as_str().unwrap();

        match kind {
            "OBJECT" => {
                let fields = object["fields"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|field_def| Field::from_json_value(field_def))
                    .collect();

                Some(Type::Object(ObjectType { name, fields }))
            }
            "INPUT_OBJECT" => {
                let args_elems = object["inputFields"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|arg_def| Arg::from_json_value(arg_def))
                    .collect();
                let args = ArgList { elems: args_elems };

                Some(Type::InputObject(InputObjectType { name, args }))
            }
            _ => None,
        }
    }

    pub fn field_names(&self, prefix: String) -> Vec<String> {
        match self {
            Type::Object(object_type) => object_type
                .fields
                .iter()
                .filter_map(|field| {
                    if field.name.starts_with(&prefix) {
                        Some(field.name.clone())
                    } else {
                        None
                    }
                })
                .collect(),
            Type::InputObject(input_object) => input_object
                .args
                .elems
                .iter()
                .map(|arg| arg.name.clone())
                .collect(),
        }
    }

    pub fn field(&self, name: String) -> Option<&Field> {
        match self {
            Type::Object(object_type) => {
                for field in &object_type.fields {
                    if field.name == name {
                        return Some(field);
                    }
                }
                None
            }
            Type::InputObject(_) => None,
        }
    }
}

pub struct Schema {
    types: Vec<Type>,
    pub query_root_name: String,
    pub mutation_root_name: String,
}

impl Schema {
    pub fn new() -> Schema {
        let schema: Value =
            serde_json::from_reader(File::open("./misc/shopify.json").unwrap()).unwrap();

        let query_root_name = schema.as_object().unwrap()["data"].as_object().unwrap()["__schema"]
            .as_object()
            .unwrap()["queryType"]
            .as_object()
            .unwrap()["name"]
            .as_str()
            .unwrap()
            .to_string();

        let mutation_root_name = schema.as_object().unwrap()["data"].as_object().unwrap()
            ["__schema"]
            .as_object()
            .unwrap()["mutationType"]
            .as_object()
            .unwrap()["name"]
            .as_str()
            .unwrap()
            .to_string();

        Schema {
            types: Schema::read_types(&schema),
            query_root_name,
            mutation_root_name,
        }
    }

    pub fn type_definition(&self, name: String) -> Option<&Type> {
        for ty in &self.types {
            match ty {
                Type::Object(object_type) => {
                    if object_type.name == name {
                        return Some(ty);
                    }
                }
                Type::InputObject(input_object) => {
                    if input_object.name == name {
                        return Some(ty);
                    }
                }
            }
        }

        None
    }

    fn read_types(schema: &Value) -> Vec<Type> {
        schema.as_object().unwrap()["data"].as_object().unwrap()["__schema"]
            .as_object()
            .unwrap()["types"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|type_def| Type::from_json_value(type_def))
            .collect()
    }

    pub fn field_type(&self, type_definition: &Type, field_name: String) -> Result<&Type, String> {
        type_definition
            .field(field_name.clone())
            .ok_or(format!("Field {} not found", field_name))
            .and_then(|field_definition| {
                field_definition
                    .field_type
                    .underlying_type_name()
                    .ok_or(format!("Field type of {} not found", field_name))
            })
            .and_then(|field_type_name| {
                self.type_definition(field_type_name.clone()).ok_or(format!(
                    "Definition of type {} of field {} not found",
                    field_type_name, field_name
                ))
            })
    }
}
