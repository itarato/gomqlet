use std::ops::{Range, RangeInclusive};

use crate::tokenizer::Token;

pub enum Root {
    Query(Query),
    Mutation(Mutation),
}

#[derive(Debug)]
pub struct Query {
    pub field_list: FieldList,
}

pub struct Mutation {
    pub field_list: FieldList,
}

#[derive(Debug)]
pub struct FieldList {
    pub start_pos: usize,
    pub end_pos: usize,
    pub fields: Vec<Field>,
}

impl FieldList {
    pub fn new_empty(pos: usize) -> FieldList {
        FieldList {
            start_pos: pos,
            end_pos: pos,
            fields: vec![],
        }
    }

    pub fn range_exclusive(&self) -> Range<usize> {
        self.start_pos..self.end_pos
    }
}

#[derive(Debug)]
pub enum Field {
    Concrete(ConcreteField),
    Union(UnionField),
}

impl Field {
    pub fn range_inclusive(&self) -> RangeInclusive<usize> {
        match self {
            Field::Concrete(field) => field.start_pos..=field.end_pos,
            Field::Union(field) => field.start_pos..=field.end_pos,
        }
    }

    #[cfg(test)]
    pub fn as_concrete_field(&self) -> &ConcreteField {
        match &self {
            Field::Concrete(field) => field,
            _ => panic!("Expected to be a concrete field"),
        }
    }

    #[cfg(test)]
    pub fn as_union_field(&self) -> &UnionField {
        match &self {
            Field::Union(field) => field,
            _ => panic!("Expected to be a concrete field"),
        }
    }
}

#[derive(Debug)]
pub struct ConcreteField {
    pub start_pos: usize,
    pub end_pos: usize,
    pub name: Token,
    pub arglist: Option<ArgList>,
    pub field_list: Option<FieldList>,
}

#[derive(Debug)]
pub struct UnionField {
    pub start_pos: usize,
    pub end_pos: usize,
    pub type_name: Token,
    pub field_list: FieldList,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ArgList {
    pub start_pos: usize,
    pub end_pos: usize,
    pub params: Vec<ParamKeyValuePair>,
}

impl ArgList {
    pub fn range_exclusive(&self) -> Range<usize> {
        self.start_pos..self.end_pos
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParamKeyValuePair {
    pub start_pos: usize,
    pub end_pos: usize,
    pub key: Token,
    pub value: ParamValue,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParamValue {
    Simple(Token),
    List(ListParamValue),
    Object(ArgList),
    // For error correction reasons a placeholder type.
    Missing((usize, usize)), // (start_pos, end_pos)
}

impl ParamValue {
    pub fn start_pos(&self) -> usize {
        match self {
            ParamValue::Simple(token) => token.pos,
            ParamValue::Missing((start_pos, _)) => *start_pos,
            ParamValue::List(list) => list.start_pos,
            ParamValue::Object(object) => object.start_pos,
        }
    }

    pub fn end_pos(&self) -> usize {
        match self {
            ParamValue::Simple(token) => token.end_pos(),
            ParamValue::Missing((_, end_pos)) => *end_pos,
            ParamValue::List(list) => list.end_pos,
            ParamValue::Object(object) => object.end_pos,
        }
    }

    #[allow(unused)]
    pub fn as_list(&self) -> &ListParamValue {
        match &self {
            ParamValue::List(list) => list,
            _ => panic!("Param value expected to be a list"),
        }
    }

    #[allow(unused)]
    pub fn as_object(&self) -> &ArgList {
        match &self {
            ParamValue::Object(object) => object,
            _ => panic!("Param value expected to be an object"),
        }
    }

    #[allow(unused)]
    pub fn as_simple(&self) -> &Token {
        match &self {
            ParamValue::Simple(token) => token,
            _ => panic!("Param value expected to be a simple type"),
        }
    }

    pub fn range_inclusive(&self) -> RangeInclusive<usize> {
        self.start_pos()..=self.end_pos()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ListParamValue {
    pub start_pos: usize,
    pub end_pos: usize,
    pub elems: Vec<ParamValue>,
}
