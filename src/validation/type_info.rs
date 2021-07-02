//! TypeInfo is a utility class which, given a GraphQL schema, can keep track
//! of the current field and type definitions at any point in a GraphQL document
//! AST during a recursive descent by calling `enter(node)` and `leave(node)`.

use crate::{
    common::{CompositeType, InputType},
    query::Field,
    schema::{Directive, EnumValue, SchemaDefinition, Text, Type, Value},
};
pub struct TypeInfo<'ast, T: Text<'ast>> {
    schema: SchemaDefinition<'ast, T>,
    type_stack: Vec<Option<Type<'ast, T>>>,
    parent_type_stack: Vec<Option<CompositeType<'ast, T>>>,
    input_type_stack: Vec<Option<InputType<'ast, T>>>,
    field_def_stack: Vec<Option<Field<'ast, T>>>,
    default_value_stack: Vec<Option<T>>,
    directive: Option<Directive<'ast, T>>,
    argument: Option<(T::Value, Value<'ast, T>)>,
    enum_value: Option<EnumValue<'ast, T>>,
}

impl<'ast, T: Text<'ast>> TypeInfo<'ast, T> {
    pub fn new(schema: SchemaDefinition<'ast, T>) -> TypeInfo<'ast, T> {
        TypeInfo {
            schema,
            type_stack: Vec::new(),
            parent_type_stack: Vec::new(),
            input_type_stack: Vec::new(),
            field_def_stack: Vec::new(),
            default_value_stack: Vec::new(),
            directive: None,
            argument: None,
            enum_value: None,
        }
    }
}
