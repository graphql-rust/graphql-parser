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
//!     walk_document(&mut number_of_type, &doc);
//!
//!     assert_eq!(number_of_type.count, 2);
//! }
//! ```
//!
//! [`QueryVisitor`]: /graphql_parser/query/query_visitor/trait.QueryVisitor.html
//! [`walk_document`]: /graphql_parser/query/query_visitor/fn.walk_document.html

#![allow(unused_variables)]

use super::ast::*;

/// Trait for easy query syntax tree traversal.
///
/// See [module docs](/graphql_parser/query/query_visitor/index.html) for more info.
pub trait QueryVisitor<'ast> {
    fn visit_document(&mut self, node: &'ast Document) {}

    fn visit_definition(&mut self, node: &'ast Definition) {}

    fn visit_fragment_definition(&mut self, node: &'ast FragmentDefinition) {}

    fn visit_operation_definition(&mut self, node: &'ast OperationDefinition) {}

    fn visit_query(&mut self, node: &'ast Query) {}

    fn visit_mutation(&mut self, node: &'ast Mutation) {}

    fn visit_subscription(&mut self, node: &'ast Subscription) {}

    fn visit_selection_set(&mut self, node: &'ast SelectionSet) {}

    fn visit_variable_definition(&mut self, node: &'ast VariableDefinition) {}

    fn visit_selection(&mut self, node: &'ast Selection) {}

    fn visit_field(&mut self, node: &'ast Field) {}

    fn visit_fragment_spread(&mut self, node: &'ast FragmentSpread) {}

    fn visit_inline_fragment(&mut self, node: &'ast InlineFragment) {}
}


/// Walk a query syntax tree and call the visitor methods for each type of node.
///
/// This function is how you should initiate a visitor.
pub fn walk_document<'ast, V: QueryVisitor<'ast>>(visitor: &mut V, node: &'ast Document) {
    visitor.visit_document(node);
    for def in &node.definitions {
        walk_definition(visitor, def);
    }
}

fn walk_definition<'ast, V: QueryVisitor<'ast>>(visitor: &mut V, node: &'ast Definition) {
    use super::ast::Definition::*;

    visitor.visit_definition(node);
    match node {
        Operation(inner) => {
            visitor.visit_operation_definition(inner);
            walk_operation_definition(visitor, inner);
        },
        Fragment(inner) => {
            visitor.visit_fragment_definition(inner);
            walk_fragment_definition(visitor, inner);
        },
    }
}

fn walk_fragment_definition<'ast, V: QueryVisitor<'ast>>(visitor: &mut V, node: &'ast FragmentDefinition) {
    visitor.visit_fragment_definition(node);
}

fn walk_operation_definition<'ast, V: QueryVisitor<'ast>>(visitor: &mut V, node: &'ast OperationDefinition) {
    use super::ast::OperationDefinition::*;

    visitor.visit_operation_definition(node);
    match node {
        SelectionSet(inner) => {
            visitor.visit_selection_set(inner);
            walk_selection_set(visitor, inner);
        }
        Query(inner) => {
            visitor.visit_query(inner);
            walk_query(visitor, inner);
        }
        Mutation(inner) => {
            visitor.visit_mutation(inner);
            walk_mutation(visitor, inner);
        }
        Subscription(inner) => {
            visitor.visit_subscription(inner);
            walk_subscription(visitor, inner);
        }
    }
}

fn walk_query<'ast, V: QueryVisitor<'ast>>(visitor: &mut V, node: &'ast Query) {
    visitor.visit_query(node);

    for var_def in &node.variable_definitions {
        visitor.visit_variable_definition(var_def);
        walk_variable_definition(visitor, var_def);
    }

    visitor.visit_selection_set(&node.selection_set);
    walk_selection_set(visitor, &node.selection_set);
}

fn walk_mutation<'ast, V: QueryVisitor<'ast>>(visitor: &mut V, node: &'ast Mutation) {
    visitor.visit_mutation(node);

    for var_def in &node.variable_definitions {
        visitor.visit_variable_definition(var_def);
        walk_variable_definition(visitor, var_def);
    }

    visitor.visit_selection_set(&node.selection_set);
    walk_selection_set(visitor, &node.selection_set);
}

fn walk_subscription<'ast, V: QueryVisitor<'ast>>(visitor: &mut V, node: &'ast Subscription) {
    visitor.visit_subscription(node);

    for var_def in &node.variable_definitions {
        visitor.visit_variable_definition(var_def);
        walk_variable_definition(visitor, var_def);
    }

    visitor.visit_selection_set(&node.selection_set);
    walk_selection_set(visitor, &node.selection_set);
}

fn walk_selection_set<'ast, V: QueryVisitor<'ast>>(visitor: &mut V, node: &'ast SelectionSet) {
    visitor.visit_selection_set(node);

    for selection in &node.items {
        visitor.visit_selection(selection);
        walk_selection(visitor, selection);
    }
}

fn walk_variable_definition<'ast, V: QueryVisitor<'ast>>(visitor: &mut V, node: &'ast VariableDefinition) {
    visitor.visit_variable_definition(node)
}

fn walk_selection<'ast, V: QueryVisitor<'ast>>(visitor: &mut V, node: &'ast Selection) {
    use super::ast::Selection::*;

    visitor.visit_selection(node);
    match node {
        Field(inner) => {
            visitor.visit_field(inner);
            walk_field(visitor, inner);
        }
        FragmentSpread(inner) => {
            visitor.visit_fragment_spread(inner);
            walk_fragment_spread(visitor, inner);
        }
        InlineFragment(inner) => {
            visitor.visit_inline_fragment(inner);
            walk_inline_fragment(visitor, inner);
        }
    }
}

fn walk_field<'ast, V: QueryVisitor<'ast>>(visitor: &mut V, node: &'ast Field) {
    visitor.visit_field(node)
}

fn walk_fragment_spread<'ast, V: QueryVisitor<'ast>>(visitor: &mut V, node: &'ast FragmentSpread) {
    visitor.visit_fragment_spread(node)
}

fn walk_inline_fragment<'ast, V: QueryVisitor<'ast>>(visitor: &mut V, node: &'ast InlineFragment) {
    visitor.visit_inline_fragment(node);
}
