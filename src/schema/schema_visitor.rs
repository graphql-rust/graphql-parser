//! Schema syntax tree traversal.
//!
//! Each method of [`SchemaVisitor`] is a hook that can be overridden to customize the behavior when
//! visiting the corresponding type of node. By default, the methods don't do anything. The actual
//! walking of the ast is done by the `walk_*` functions. So to run a visitor over the whole
//! document you should use [`walk_document`].
//!
//! Example:
//!
//! ```
//! use graphql_parser::schema::{
//!     ObjectType,
//!     parse_schema,
//!     schema_visitor::{SchemaVisitor, walk_document},
//! };
//!
//! struct ObjectTypesCounter {
//!     count: usize,
//! }
//!
//! impl ObjectTypesCounter {
//!     fn new() -> Self {
//!         Self { count: 0 }
//!     }
//! }
//!
//! impl<'ast> SchemaVisitor<'ast> for ObjectTypesCounter {
//!     fn visit_object_type(&mut self, node: &'ast ObjectType) {
//!         self.count += 1;
//!     }
//! }
//!
//! fn main() {
//!     let mut number_of_type = ObjectTypesCounter::new();
//!
//!     let doc = parse_schema(r#"
//!         schema {
//!             query: Query
//!         }
//!
//!         type Query {
//!             users: [User!]!
//!         }
//!
//!         type User {
//!             id: ID!
//!         }
//!     "#).expect("Failed to parse schema");
//!
//!     walk_document(&mut number_of_type, &doc);
//!
//!     assert_eq!(number_of_type.count, 2);
//! }
//! ```
//!
//! [`SchemaVisitor`]: /graphql_parser/schema/schema_visitor/trait.SchemaVisitor.html
//! [`walk_document`]: /graphql_parser/schema/schema_visitor/fn.walk_document.html

#![allow(unused_variables)]

use super::ast::*;

/// Trait for easy schema syntax tree traversal.
///
/// See [module docs](/graphql_parser/schema/schema_visitor/index.html) for more info.
pub trait SchemaVisitor<'ast> {
    fn visit_document(&mut self, doc: &'ast Document) {}

    fn visit_schema_definition(&mut self, node: &'ast SchemaDefinition) {}

    fn visit_directive_definition(&mut self, node: &'ast DirectiveDefinition) {}

    fn visit_type_definition(&mut self, node: &'ast TypeDefinition) {}

    fn visit_scalar_type(&mut self, node: &'ast ScalarType) {}

    fn visit_object_type(&mut self, node: &'ast ObjectType) {}

    fn visit_interface_type(&mut self, node: &'ast InterfaceType) {}

    fn visit_union_type(&mut self, node: &'ast UnionType) {}

    fn visit_enum_type(&mut self, node: &'ast EnumType) {}

    fn visit_input_object_type(&mut self, node: &'ast InputObjectType) {}

    fn visit_type_extension(&mut self, node: &'ast TypeExtension) {}

    fn visit_scalar_type_extension(&mut self, node: &'ast ScalarTypeExtension) {}

    fn visit_object_type_extension(&mut self, node: &'ast ObjectTypeExtension) {}

    fn visit_interface_type_extension(&mut self, node: &'ast InterfaceTypeExtension) {}

    fn visit_union_type_extension(&mut self, node: &'ast UnionTypeExtension) {}

    fn visit_enum_type_extension(&mut self, node: &'ast EnumTypeExtension) {}

    fn visit_input_object_type_extension(&mut self, node: &'ast InputObjectTypeExtension) {}
}

/// Walk a schema syntax tree and call the visitor methods for each type of node.
///
/// This function is how you should initiate a visitor.
pub fn walk_document<'ast, V: SchemaVisitor<'ast>>(visitor: &mut V, doc: &'ast Document) {
    use super::ast::Definition::*;

    for def in &doc.definitions {
        match def {
            SchemaDefinition(inner) => {
                visitor.visit_schema_definition(inner);
                walk_schema_definition(visitor, inner);
            }
            TypeDefinition(inner) => {
                visitor.visit_type_definition(inner);
                walk_type_definition(visitor, inner);
            }
            TypeExtension(inner) => {
                visitor.visit_type_extension(inner);
                walk_type_extension(visitor, inner);
            }
            DirectiveDefinition(inner) => {
                visitor.visit_directive_definition(inner);
                walk_directive_definition(visitor, inner);
            }
        }
    }
}

fn walk_schema_definition<'ast, V: SchemaVisitor<'ast>>(visitor: &mut V, node: &'ast SchemaDefinition) {}

fn walk_directive_definition<'ast, V: SchemaVisitor<'ast>>(visitor: &mut V, node: &'ast DirectiveDefinition) {}

fn walk_type_definition<'ast, V: SchemaVisitor<'ast>>(visitor: &mut V, node: &'ast TypeDefinition) {
    use super::ast::TypeDefinition::*;

    match node {
        Scalar(inner) => {
            visitor.visit_scalar_type(inner);
            walk_scalar_type(visitor, inner);
        }
        Object(inner) => {
            visitor.visit_object_type(inner);
            walk_object_type(visitor, inner);
        }
        Interface(inner) => {
            visitor.visit_interface_type(inner);
            walk_interface_type(visitor, inner);
        }
        Union(inner) => {
            visitor.visit_union_type(inner);
            walk_union_type(visitor, inner);
        }
        Enum(inner) => {
            visitor.visit_enum_type(inner);
            walk_enum_type(visitor, inner);
        }
        InputObject(inner) => {
            visitor.visit_input_object_type(inner);
            walk_input_object_type(visitor, inner);
        }
    }
}

fn walk_scalar_type<'ast, V: SchemaVisitor<'ast>>(visitor: &mut V, node: &'ast ScalarType) {}

fn walk_object_type<'ast, V: SchemaVisitor<'ast>>(visitor: &mut V, node: &'ast ObjectType) {}

fn walk_interface_type<'ast, V: SchemaVisitor<'ast>>(visitor: &mut V, node: &'ast InterfaceType) {}

fn walk_union_type<'ast, V: SchemaVisitor<'ast>>(visitor: &mut V, node: &'ast UnionType) {}

fn walk_enum_type<'ast, V: SchemaVisitor<'ast>>(visitor: &mut V, node: &'ast EnumType) {}

fn walk_input_object_type<'ast, V: SchemaVisitor<'ast>>(visitor: &mut V, node: &'ast InputObjectType) {}

fn walk_type_extension<'ast, V: SchemaVisitor<'ast>>(visitor: &mut V, node: &'ast TypeExtension) {
    use super::ast::TypeExtension::*;

    match node {
        Scalar(inner) => {
            visitor.visit_scalar_type_extension(inner);
            walk_scalar_type_extension(visitor, inner);
        }
        Object(inner) => {
            visitor.visit_object_type_extension(inner);
            walk_object_type_extension(visitor, inner);
        }
        Interface(inner) => {
            visitor.visit_interface_type_extension(inner);
            walk_interface_type_extension(visitor, inner);
        }
        Union(inner) => {
            visitor.visit_union_type_extension(inner);
            walk_union_type_extension(visitor, inner);
        }
        Enum(inner) => {
            visitor.visit_enum_type_extension(inner);
            walk_enum_type_extension(visitor, inner);
        }
        InputObject(inner) => {
            visitor.visit_input_object_type_extension(inner);
            walk_input_object_type_extension(visitor, inner);
        }
    }
}

fn walk_scalar_type_extension<'ast, V: SchemaVisitor<'ast>>(visitor: &mut V, node: &'ast ScalarTypeExtension) {}

fn walk_object_type_extension<'ast, V: SchemaVisitor<'ast>>(visitor: &mut V, node: &'ast ObjectTypeExtension) {}

fn walk_interface_type_extension<'ast, V: SchemaVisitor<'ast>>(visitor: &mut V, node: &'ast InterfaceTypeExtension) {}

fn walk_union_type_extension<'ast, V: SchemaVisitor<'ast>>(visitor: &mut V, node: &'ast UnionTypeExtension) {}

fn walk_enum_type_extension<'ast, V: SchemaVisitor<'ast>>(visitor: &mut V, node: &'ast EnumTypeExtension) {}

fn walk_input_object_type_extension<'ast, V: SchemaVisitor<'ast>>(visitor: &mut V, node: &'ast InputObjectTypeExtension) {}
