use std::fs::File;

use serde_json::Value;

pub enum FieldType {
    NonNull(Box<FieldType>),
    List(Box<FieldType>),
    Object(String),
    Enum(String),
    Interface(String),
    Scalar,
    Input(String),
    Union(String),
}

pub struct Field {
    pub name: String,
    pub field_type: FieldType,
}

impl Field {
    fn from_json_value(node: Value) -> Field {}
}

pub struct ObjectType {
    name: String,
    fields: Vec<Field>,
}

pub enum Type {
    Object(ObjectType),
}

impl Type {
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
            .as_str()
            .unwrap()
            .to_string();

        let mutation_root_name = schema.as_object().unwrap()["data"].as_object().unwrap()
            ["__schema"]
            .as_object()
            .unwrap()["mutationType"]
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
                _ => continue,
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
            .filter_map(|type_def| {
                let object_type_def = type_def.as_object().unwrap();

                let name = object_type_def["name"].as_str().unwrap().to_string();
                match object_type_def["kind"].as_str().unwrap() {
                    "OBJECT" => {
                        let fields = object_type_def["fields"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|field_def| {
                                let field_name = field_def.as_object().unwrap()["name"].to_string();
                                Field { name: field_name }
                            })
                            .collect();

                        Some(Type::Object(ObjectType { name, fields }))
                    }
                    _ => None,
                }
            })
            .collect()
    }

    fn field_type_defition_of_parent_type_definition(
        &self,
        type_definition: &Type,
        field_name: String,
    ) -> Result<Type, String> {
        type_definition
            .field(field_name.clone())
            .ok_or(format!("Field {} not found", field_name))
            .and_then(|field_definition| {
                Analyzer::lookup_type_name_from_field_definition(field_definition)
                    .ok_or(format!("Field {} not found", field_name))
            })
            .and_then(|field_type_name| {
                Analyzer::lookup_graphql_type_definition(schema, field_type_name.clone()).ok_or(
                    format!(
                        "Definition of type {} of field {} not found",
                        field_type_name, field_name
                    ),
                )
            })
    }

    fn lookup_type_name_from_type<'a>(ty: &'a Type<'a, String>) -> Option<String> {
        match &ty {
            Type::NamedType(name) => Some(name.clone()),
            Type::ListType(list) => Analyzer::lookup_type_name_from_type(list),
            Type::NonNullType(non_null_ty) => Analyzer::lookup_type_name_from_type(non_null_ty),
        }
    }
}
