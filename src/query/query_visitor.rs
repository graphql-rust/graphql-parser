//! Query syntax tree traversal.
//!
//! Each method of [`QueryVisitor`] is a hook that can be overridden to customize the behavior when
//! visiting the corresponding type of node. By default, the methods don't do anything. The actual
//! walking of the ast is done by the `walk_*` functions. So to run a visitor over the whole
//! document you should use [`walk_document`].
//!
//! Example:
//!
//! ```
//! use graphql_parser::query::{
//!     Field,
//!     parse_query,
//!     query_visitor::{QueryVisitor, walk_document},
//! };
//!
//! struct FieldsCounter {
//!     count: usize,
//! }
//!
//! impl FieldsCounter {
//!     fn new() -> Self {
//!         Self { count: 0 }
//!     }
//! }
//!
//! impl<'ast> QueryVisitor<'ast> for FieldsCounter {
//!     fn visit_field(&mut self, node: &'ast Field) {
//!         self.count += 1
//!     }
//! }
//!
//! fn main() {
//!     let mut number_of_type = FieldsCounter::new();
//!
//!     let doc = parse_query(r#"
//!         query TestQuery {
//!             users {
//!                 id
//!                 country {
//!                     id
//!                 }
//!             }
//!         }
//!     "#).expect("Failed to parse query");
//!
//!     walk_document(&mut number_of_type, &doc, None);
//!
//!     assert_eq!(number_of_type.count, 2);
//! }
//! ```
//!
//! [`QueryVisitor`]: /graphql_parser/query/query_visitor/trait.QueryVisitor.html
//! [`walk_document`]: /graphql_parser/query/query_visitor/fn.walk_document.html

#![allow(unused_variables)]

use super::ast::*;
use crate::validation::TypeInfo;

/// Trait for easy query syntax tree traversal.
///
/// See [module docs](/graphql_parser/query/query_visitor/index.html) for more info.
pub trait QueryVisitor<'ast, T: Text<'ast>> {
    fn visit_document(
        &mut self,
        node: &'ast Document<'ast, T>,
        type_info: &mut Option<TypeInfo<'ast, T>>,
    ) {
    }

    fn visit_definition(
        &mut self,
        node: &'ast Definition<'ast, T>,
        type_info: &mut Option<TypeInfo<'ast, T>>,
    ) {
    }

    fn visit_fragment_definition(
        &mut self,
        node: &'ast FragmentDefinition<'ast, T>,
        type_info: &mut Option<TypeInfo<'ast, T>>,
    ) {
    }

    fn visit_operation_definition(
        &mut self,
        node: &'ast OperationDefinition<'ast, T>,
        type_info: &mut Option<TypeInfo<'ast, T>>,
    ) {
    }

    fn visit_query(
        &mut self,
        node: &'ast Query<'ast, T>,
        type_info: &mut Option<TypeInfo<'ast, T>>,
    ) {
    }

    fn visit_mutation(
        &mut self,
        node: &'ast Mutation<'ast, T>,
        type_info: &mut Option<TypeInfo<'ast, T>>,
    ) {
    }

    fn visit_subscription(
        &mut self,
        node: &'ast Subscription<'ast, T>,
        type_info: &mut Option<TypeInfo<'ast, T>>,
    ) {
    }

    fn visit_selection_set(
        &mut self,
        node: &'ast SelectionSet<'ast, T>,
        type_info: &mut Option<TypeInfo<'ast, T>>,
    ) {
    }

    fn visit_variable_definition(
        &mut self,
        node: &'ast VariableDefinition<'ast, T>,
        type_info: &mut Option<TypeInfo<'ast, T>>,
    ) {
    }

    fn visit_selection(
        &mut self,
        node: &'ast Selection<'ast, T>,
        type_info: &mut Option<TypeInfo<'ast, T>>,
    ) {
    }

    fn visit_field(
        &mut self,
        node: &'ast Field<'ast, T>,
        type_info: &mut Option<TypeInfo<'ast, T>>,
    ) {
    }

    fn visit_fragment_spread(
        &mut self,
        node: &'ast FragmentSpread<'ast, T>,
        type_info: &mut Option<TypeInfo<'ast, T>>,
    ) {
    }

    fn visit_inline_fragment(
        &mut self,
        node: &'ast InlineFragment<'ast, T>,
        type_info: &mut Option<TypeInfo<'ast, T>>,
    ) {
    }
}

/// Walk a query syntax tree and call the visitor methods for each type of node.
///
/// This function is how you should initiate a visitor.
pub fn walk_document<'ast, T: Text<'ast>, V: QueryVisitor<'ast, T>>(
    visitor: &mut V,
    node: &'ast Document<'ast, T>,
    type_info: &mut Option<TypeInfo<'ast, T>>,
) {
    if let Some(info) = type_info {
        info.visit_document(node, &mut None);
    }

    visitor.visit_document(node, type_info);

    for def in &node.definitions {
        walk_definition(visitor, def, type_info);
    }
}

fn walk_definition<'ast, T: Text<'ast>, V: QueryVisitor<'ast, T>>(
    visitor: &mut V,
    node: &'ast Definition<'ast, T>,
    type_info: &mut Option<TypeInfo<'ast, T>>,
) {
    use super::ast::Definition::*;

    if let Some(info) = type_info {
        info.visit_definition(node, &mut None);
    }

    visitor.visit_definition(node, type_info);

    match node {
        Operation(inner) => {
            visitor.visit_operation_definition(inner, type_info);
            walk_operation_definition(visitor, inner, type_info);
        }
        Fragment(inner) => {
            visitor.visit_fragment_definition(inner, type_info);
            walk_fragment_definition(visitor, inner, type_info);
        }
    }
}

fn walk_fragment_definition<'ast, T: Text<'ast>, V: QueryVisitor<'ast, T>>(
    visitor: &mut V,
    node: &'ast FragmentDefinition<'ast, T>,
    type_info: &mut Option<TypeInfo<'ast, T>>,
) {
    if let Some(info) = type_info {
        info.visit_fragment_definition(node, &mut None);
    }

    visitor.visit_fragment_definition(node, type_info);

    walk_selection_set(visitor, &node.selection_set, type_info);
}

fn walk_operation_definition<'ast, T: Text<'ast>, V: QueryVisitor<'ast, T>>(
    visitor: &mut V,
    node: &'ast OperationDefinition<'ast, T>,
    type_info: &mut Option<TypeInfo<'ast, T>>,
) {
    use super::ast::OperationDefinition::*;

    if let Some(info) = type_info {
        info.visit_operation_definition(node, &mut None);
    }

    visitor.visit_operation_definition(node, type_info);

    match node {
        SelectionSet(inner) => {
            visitor.visit_selection_set(inner, type_info);
            walk_selection_set(visitor, inner, type_info);
        }
        Query(inner) => {
            visitor.visit_query(inner, type_info);
            walk_query(visitor, inner, type_info);
        }
        Mutation(inner) => {
            visitor.visit_mutation(inner, type_info);
            walk_mutation(visitor, inner, type_info);
        }
        Subscription(inner) => {
            visitor.visit_subscription(inner, type_info);
            walk_subscription(visitor, inner, type_info);
        }
    }
}

fn walk_query<'ast, T: Text<'ast>, V: QueryVisitor<'ast, T>>(
    visitor: &mut V,
    node: &'ast Query<'ast, T>,
    type_info: &mut Option<TypeInfo<'ast, T>>,
) {
    if let Some(info) = type_info {
        info.visit_query(node, &mut None);
    }

    visitor.visit_query(node, type_info);

    for var_def in &node.variable_definitions {
        visitor.visit_variable_definition(var_def, type_info);
        walk_variable_definition(visitor, var_def, type_info);
    }

    visitor.visit_selection_set(&node.selection_set, type_info);
    walk_selection_set(visitor, &node.selection_set, type_info);
}

fn walk_mutation<'ast, T: Text<'ast>, V: QueryVisitor<'ast, T>>(
    visitor: &mut V,
    node: &'ast Mutation<'ast, T>,
    type_info: &mut Option<TypeInfo<'ast, T>>,
) {
    if let Some(info) = type_info {
        info.visit_mutation(node, &mut None);
    }

    visitor.visit_mutation(node, type_info);

    for var_def in &node.variable_definitions {
        visitor.visit_variable_definition(var_def, type_info);
        walk_variable_definition(visitor, var_def, type_info);
    }

    visitor.visit_selection_set(&node.selection_set, type_info);
    walk_selection_set(visitor, &node.selection_set, type_info);
}

fn walk_subscription<'ast, T: Text<'ast>, V: QueryVisitor<'ast, T>>(
    visitor: &mut V,
    node: &'ast Subscription<'ast, T>,
    type_info: &mut Option<TypeInfo<'ast, T>>,
) {
    if let Some(info) = type_info {
        info.visit_subscription(node, &mut None);
    }

    visitor.visit_subscription(node, type_info);

    for var_def in &node.variable_definitions {
        visitor.visit_variable_definition(var_def, type_info);
        walk_variable_definition(visitor, var_def, type_info);
    }

    visitor.visit_selection_set(&node.selection_set, type_info);
    walk_selection_set(visitor, &node.selection_set, type_info);
}

fn walk_selection_set<'ast, T: Text<'ast>, V: QueryVisitor<'ast, T>>(
    visitor: &mut V,
    node: &'ast SelectionSet<'ast, T>,
    type_info: &mut Option<TypeInfo<'ast, T>>,
) {
    if let Some(info) = type_info {
        info.visit_selection_set(node, &mut None);
    }

    visitor.visit_selection_set(node, type_info);

    for selection in &node.items {
        visitor.visit_selection(selection, type_info);
        walk_selection(visitor, selection, type_info);
    }
}

fn walk_variable_definition<'ast, T: Text<'ast>, V: QueryVisitor<'ast, T>>(
    visitor: &mut V,
    node: &'ast VariableDefinition<'ast, T>,
    type_info: &mut Option<TypeInfo<'ast, T>>,
) {
    if let Some(info) = type_info {
        info.visit_variable_definition(node, &mut None);
    }

    visitor.visit_variable_definition(node, type_info);
}

fn walk_selection<'ast, T: Text<'ast>, V: QueryVisitor<'ast, T>>(
    visitor: &mut V,
    node: &'ast Selection<'ast, T>,
    type_info: &mut Option<TypeInfo<'ast, T>>,
) {
    use super::ast::Selection::*;

    if let Some(info) = type_info {
        info.visit_selection(node, &mut None);
    }

    visitor.visit_selection(node, type_info);

    match node {
        Field(inner) => {
            visitor.visit_field(inner, type_info);
            walk_field(visitor, inner, type_info);
        }
        FragmentSpread(inner) => {
            visitor.visit_fragment_spread(inner, type_info);
            walk_fragment_spread(visitor, inner, type_info);
        }
        InlineFragment(inner) => {
            visitor.visit_inline_fragment(inner, type_info);
            walk_inline_fragment(visitor, inner, type_info);
        }
    }
}

fn walk_field<'ast, T: Text<'ast>, V: QueryVisitor<'ast, T>>(
    visitor: &mut V,
    node: &'ast Field<'ast, T>,
    type_info: &mut Option<TypeInfo<'ast, T>>,
) {
    if let Some(info) = type_info {
        info.visit_field(node, &mut None);
    }

    visitor.visit_field(node, type_info);
}

fn walk_fragment_spread<'ast, T: Text<'ast>, V: QueryVisitor<'ast, T>>(
    visitor: &mut V,
    node: &'ast FragmentSpread<'ast, T>,
    type_info: &mut Option<TypeInfo<'ast, T>>,
) {
    if let Some(info) = type_info {
        info.visit_fragment_spread(node, &mut None);
    }

    visitor.visit_fragment_spread(node, type_info);
}

fn walk_inline_fragment<'ast, T: Text<'ast>, V: QueryVisitor<'ast, T>>(
    visitor: &mut V,
    node: &'ast InlineFragment<'ast, T>,
    type_info: &mut Option<TypeInfo<'ast, T>>,
) {
    if let Some(info) = type_info {
        info.visit_inline_fragment(node, &mut None);
    }

    visitor.visit_inline_fragment(node, type_info);

    walk_selection_set(visitor, &node.selection_set, type_info);
}
