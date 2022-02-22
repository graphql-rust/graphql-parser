#![allow(unused_variables)]

use super::ast::{
    Definition, Directive, Document, Field, FragmentDefinition, FragmentSpread, InlineFragment,
    Mutation, OperationDefinition, Query, Selection, SelectionSet, Subscription, Text, Value,
    VariableDefinition,
};

#[derive(Clone, Debug)]
pub enum Transformed<T> {
    Keep,
    Replace(T),
}

#[derive(Clone, Debug)]
pub enum TransformedValue<T> {
    Keep,
    Replace(T),
}

impl<T> TransformedValue<T> {
    pub fn should_keep(&self) -> bool {
        match self {
            TransformedValue::Keep => true,
            TransformedValue::Replace(_) => false,
        }
    }

    pub fn replace_or_else<F>(self, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        match self {
            TransformedValue::Keep => f(),
            TransformedValue::Replace(next_value) => next_value,
        }
    }
}

impl<T> Into<Transformed<T>> for TransformedValue<T> {
    fn into(self) -> Transformed<T> {
        match self {
            TransformedValue::Keep => Transformed::Keep,
            TransformedValue::Replace(replacement) => Transformed::Replace(replacement),
        }
    }
}

pub trait Transformer<'a, T: Text<'a> + Clone> {
    fn transform_document(
        &mut self,
        document: &Document<'a, T>,
    ) -> TransformedValue<Document<'a, T>> {
        self.default_transform_document(document)
    }

    fn default_transform_document(
        &mut self,
        document: &Document<'a, T>,
    ) -> TransformedValue<Document<'a, T>> {
        let mut next_document = Document {
            definitions: Vec::new(),
        };
        let mut has_changes = false;
        for definition in document.definitions.clone() {
            match self.transform_definition(&definition) {
                Transformed::Keep => next_document.definitions.push(definition),
                Transformed::Replace(replacement) => {
                    has_changes = true;
                    next_document.definitions.push(replacement)
                }
            }
        }
        if has_changes {
            TransformedValue::Replace(next_document)
        } else {
            TransformedValue::Keep
        }
    }

    fn transform_definition(
        &mut self,
        definition: &Definition<'a, T>,
    ) -> Transformed<Definition<'a, T>> {
        self.default_transform_definition(definition)
    }

    fn default_transform_definition(
        &mut self,
        definition: &Definition<'a, T>,
    ) -> Transformed<Definition<'a, T>> {
        match definition {
            Definition::Operation(operation) => match self.transform_operation(operation) {
                Transformed::Keep => Transformed::Keep,
                Transformed::Replace(replacement) => {
                    Transformed::Replace(Definition::Operation(replacement))
                }
            },
            Definition::Fragment(fragment) => match self.transform_fragment(fragment) {
                Transformed::Keep => Transformed::Keep,
                Transformed::Replace(replacement) => {
                    Transformed::Replace(Definition::Fragment(replacement))
                }
            },
        }
    }

    fn transform_operation(
        &mut self,
        operation: &OperationDefinition<'a, T>,
    ) -> Transformed<OperationDefinition<'a, T>> {
        self.default_transform_operation(operation)
    }

    fn default_transform_operation(
        &mut self,
        operation: &OperationDefinition<'a, T>,
    ) -> Transformed<OperationDefinition<'a, T>> {
        match operation {
            OperationDefinition::Query(query) => match self.transform_query(query) {
                Transformed::Keep => Transformed::Keep,
                Transformed::Replace(replacement) => {
                    Transformed::Replace(OperationDefinition::Query(replacement))
                }
            },
            OperationDefinition::Mutation(mutation) => match self.transform_mutation(mutation) {
                Transformed::Keep => Transformed::Keep,
                Transformed::Replace(replacement) => {
                    Transformed::Replace(OperationDefinition::Mutation(replacement))
                }
            },
            OperationDefinition::Subscription(subscription) => {
                match self.transform_subscription(subscription) {
                    Transformed::Keep => Transformed::Keep,
                    Transformed::Replace(replacement) => {
                        Transformed::Replace(OperationDefinition::Subscription(replacement))
                    }
                }
            }
            OperationDefinition::SelectionSet(selection_set) => {
                let items = self.transform_selection_set(selection_set);

                if items.should_keep() {
                    return Transformed::Keep;
                }

                Transformed::Replace(OperationDefinition::SelectionSet(SelectionSet {
                    items: items.replace_or_else(|| selection_set.items.clone()),
                    span: selection_set.span,
                }))
            }
        }
    }

    fn transform_query(&mut self, node: &Query<'a, T>) -> Transformed<Query<'a, T>> {
        self.default_transform_query(node)
    }

    fn default_transform_query(&mut self, node: &Query<'a, T>) -> Transformed<Query<'a, T>> {
        let selections = self.transform_selection_set(&node.selection_set);
        let directives = self.transform_directives(&node.directives);
        let variable_definitions = self.transform_variable_definitions(&node.variable_definitions);

        if selections.should_keep()
            && directives.should_keep()
            && variable_definitions.should_keep()
        {
            return Transformed::Keep;
        }

        Transformed::Replace(Query {
            directives: directives.replace_or_else(|| node.directives.clone()),
            selection_set: SelectionSet {
                items: selections.replace_or_else(|| node.selection_set.items.clone()),
                span: node.selection_set.span,
            },
            variable_definitions: variable_definitions
                .replace_or_else(|| node.variable_definitions.clone()),
            position: node.position,
            name: node.name.clone(),
        })
    }

    fn transform_mutation(&mut self, node: &Mutation<'a, T>) -> Transformed<Mutation<'a, T>> {
        self.default_transform_mutation(node)
    }

    fn default_transform_mutation(
        &mut self,
        node: &Mutation<'a, T>,
    ) -> Transformed<Mutation<'a, T>> {
        let selections = self.transform_selection_set(&node.selection_set);
        let directives = self.transform_directives(&node.directives);
        let variable_definitions = self.transform_variable_definitions(&node.variable_definitions);

        if selections.should_keep()
            && directives.should_keep()
            && variable_definitions.should_keep()
        {
            return Transformed::Keep;
        }

        Transformed::Replace(Mutation {
            directives: directives.replace_or_else(|| node.directives.clone()),
            selection_set: SelectionSet {
                items: selections.replace_or_else(|| node.selection_set.items.clone()),
                span: node.selection_set.span,
            },
            variable_definitions: variable_definitions
                .replace_or_else(|| node.variable_definitions.clone()),
            position: node.position,
            name: node.name.clone(),
        })
    }

    fn transform_subscription(
        &mut self,
        node: &Subscription<'a, T>,
    ) -> Transformed<Subscription<'a, T>> {
        self.default_transform_subscription(node)
    }

    fn default_transform_subscription(
        &mut self,
        node: &Subscription<'a, T>,
    ) -> Transformed<Subscription<'a, T>> {
        let selections = self.transform_selection_set(&node.selection_set);
        let directives = self.transform_directives(&node.directives);
        let variable_definitions = self.transform_variable_definitions(&node.variable_definitions);

        if selections.should_keep()
            && directives.should_keep()
            && variable_definitions.should_keep()
        {
            return Transformed::Keep;
        }

        Transformed::Replace(Subscription {
            directives: directives.replace_or_else(|| node.directives.clone()),
            selection_set: SelectionSet {
                items: selections.replace_or_else(|| node.selection_set.items.clone()),
                span: node.selection_set.span,
            },
            variable_definitions: variable_definitions
                .replace_or_else(|| node.variable_definitions.clone()),
            position: node.position,
            name: node.name.clone(),
        })
    }

    fn transform_fragment(
        &mut self,
        fragment: &FragmentDefinition<'a, T>,
    ) -> Transformed<FragmentDefinition<'a, T>> {
        self.default_transform_fragment(fragment)
    }

    fn default_transform_fragment(
        &mut self,
        fragment: &FragmentDefinition<'a, T>,
    ) -> Transformed<FragmentDefinition<'a, T>> {
        let selections = self.transform_selection_set(&fragment.selection_set);
        let directives = self.transform_directives(&fragment.directives);

        if selections.should_keep() && directives.should_keep() {
            return Transformed::Keep;
        }

        Transformed::Replace(FragmentDefinition {
            directives: directives.replace_or_else(|| fragment.directives.clone()),
            selection_set: SelectionSet {
                items: selections.replace_or_else(|| fragment.selection_set.items.clone()),
                span: fragment.selection_set.span,
            },
            position: fragment.position,
            name: fragment.name.clone(),
            type_condition: fragment.type_condition.clone(),
        })
    }

    fn transform_selection_set(
        &mut self,
        selections: &SelectionSet<'a, T>,
    ) -> TransformedValue<Vec<Selection<'a, T>>> {
        self.transform_list(&selections.items, Self::transform_selection)
    }

    fn transform_selection(
        &mut self,
        selection: &Selection<'a, T>,
    ) -> Transformed<Selection<'a, T>> {
        self.default_transform_selection(selection)
    }

    fn default_transform_selection(
        &mut self,
        selection: &Selection<'a, T>,
    ) -> Transformed<Selection<'a, T>> {
        match selection {
            Selection::FragmentSpread(selection) => self.transform_fragment_spread(selection),
            Selection::InlineFragment(selection) => self.transform_inline_fragment(selection),
            Selection::Field(field) => self.transform_field(field),
        }
    }

    fn transform_field(&mut self, field: &Field<'a, T>) -> Transformed<Selection<'a, T>> {
        self.default_transform_field(field)
    }

    fn default_transform_field(&mut self, field: &Field<'a, T>) -> Transformed<Selection<'a, T>> {
        let selection_set = self.transform_selection_set(&field.selection_set);
        let arguments = self.transform_arguments(&field.arguments);
        let directives = self.transform_directives(&field.directives);
        if selection_set.should_keep() && arguments.should_keep() && directives.should_keep() {
            return Transformed::Keep;
        }
        Transformed::Replace(Selection::Field(Field {
            arguments: arguments.replace_or_else(|| field.arguments.clone()),
            directives: directives.replace_or_else(|| field.directives.clone()),
            selection_set: SelectionSet {
                items: selection_set.replace_or_else(|| field.selection_set.items.clone()),
                span: field.selection_set.span,
            },
            position: field.position,
            alias: field.alias.clone(),
            name: field.name.clone(),
        }))
    }

    fn transform_fragment_spread(
        &mut self,
        spread: &FragmentSpread<'a, T>,
    ) -> Transformed<Selection<'a, T>> {
        self.default_transform_fragment_spread(spread)
    }
    fn default_transform_fragment_spread(
        &mut self,
        spread: &FragmentSpread<'a, T>,
    ) -> Transformed<Selection<'a, T>> {
        let directives = self.transform_directives(&spread.directives);
        Transformed::Replace(Selection::FragmentSpread(FragmentSpread {
            directives: directives.replace_or_else(|| spread.directives.clone()),
            position: spread.position,
            fragment_name: spread.fragment_name.clone(),
        }))
    }

    fn transform_inline_fragment(
        &mut self,
        fragment: &InlineFragment<'a, T>,
    ) -> Transformed<Selection<'a, T>> {
        self.default_transform_inline_fragment(fragment)
    }

    fn default_transform_inline_fragment(
        &mut self,
        fragment: &InlineFragment<'a, T>,
    ) -> Transformed<Selection<'a, T>> {
        let selections = self.transform_selection_set(&fragment.selection_set);
        let directives = self.transform_directives(&fragment.directives);

        if selections.should_keep() && directives.should_keep() {
            return Transformed::Keep;
        }

        Transformed::Replace(Selection::InlineFragment(InlineFragment {
            position: fragment.position,
            type_condition: fragment.type_condition.clone(),
            directives: directives.replace_or_else(|| fragment.directives.clone()),
            selection_set: SelectionSet {
                span: fragment.selection_set.span,
                items: selections.replace_or_else(|| fragment.selection_set.items.clone()),
            },
        }))
    }

    fn transform_directives(
        &mut self,
        directives: &Vec<Directive<'a, T>>,
    ) -> TransformedValue<Vec<Directive<'a, T>>> {
        self.transform_list(directives, Self::transform_directive)
    }

    fn transform_directive(
        &mut self,
        directive: &Directive<'a, T>,
    ) -> Transformed<Directive<'a, T>> {
        self.default_transform_directive(directive)
    }

    fn default_transform_directive(
        &mut self,
        directive: &Directive<'a, T>,
    ) -> Transformed<Directive<'a, T>> {
        let arguments = self.transform_arguments(&directive.arguments);
        match arguments {
            TransformedValue::Keep => Transformed::Keep,
            TransformedValue::Replace(replacement) => Transformed::Replace(Directive {
                position: directive.position,
                name: directive.name.clone(),
                arguments: replacement,
            }),
        }
    }

    fn transform_arguments(
        &mut self,
        arguments: &[(T::Value, Value<'a, T>)],
    ) -> TransformedValue<Vec<(T::Value, Value<'a, T>)>> {
        self.transform_list(arguments, Self::transform_argument)
    }

    fn transform_argument(
        &mut self,
        argument: &(T::Value, Value<'a, T>),
    ) -> Transformed<(T::Value, Value<'a, T>)> {
        self.default_transform_argument(argument)
    }

    fn default_transform_argument(
        &mut self,
        argument: &(T::Value, Value<'a, T>),
    ) -> Transformed<(T::Value, Value<'a, T>)> {
        let (name, value) = argument;

        match self.transform_value(&value) {
            TransformedValue::Keep => Transformed::Keep,
            TransformedValue::Replace(replacement) => {
                Transformed::Replace((name.clone(), replacement))
            }
        }
    }

    fn transform_value(&mut self, value: &Value<'a, T>) -> TransformedValue<Value<'a, T>> {
        self.default_transform_value(value)
    }

    fn default_transform_value(&mut self, value: &Value<'a, T>) -> TransformedValue<Value<'a, T>> {
        match value {
            Value::Variable(variable) => TransformedValue::Keep,
            Value::List(items) => TransformedValue::Keep,
            Value::Object(arguments) => TransformedValue::Keep,
            Value::Null => TransformedValue::Keep,
            Value::Boolean(_) => TransformedValue::Keep,
            Value::Enum(_) => TransformedValue::Keep,
            Value::Int(_) => TransformedValue::Keep,
            Value::Float(_) => TransformedValue::Keep,
            Value::String(_) => TransformedValue::Keep,
        }
    }

    fn transform_variable_definitions(
        &mut self,
        variable_definitions: &Vec<VariableDefinition<'a, T>>,
    ) -> TransformedValue<Vec<VariableDefinition<'a, T>>> {
        self.default_transform_variable_definitions(variable_definitions)
    }

    fn default_transform_variable_definitions(
        &mut self,
        variable_definitions: &Vec<VariableDefinition<'a, T>>,
    ) -> TransformedValue<Vec<VariableDefinition<'a, T>>> {
        self.transform_list(
            variable_definitions,
            Self::default_transform_variable_definition,
        )
    }

    fn transform_variable_definition(
        &mut self,
        variable_definition: &VariableDefinition<'a, T>,
    ) -> TransformedValue<VariableDefinition<'a, T>> {
        self.default_transform_variable_definition(variable_definition)
    }

    fn default_transform_variable_definition(
        &mut self,
        variable_definition: &VariableDefinition<'a, T>,
    ) -> TransformedValue<VariableDefinition<'a, T>> {
        let default_value: Option<TransformedValue<Value<T>>> = None;

        if let Some(value) = variable_definition.default_value.clone() {
            let transformed_default_value = self.transform_value(&value);

            if transformed_default_value.should_keep() {
                return TransformedValue::Keep;
            } else {
                return TransformedValue::Replace(VariableDefinition {
                    position: variable_definition.position,
                    name: variable_definition.name.clone(),
                    var_type: variable_definition.var_type.clone(),
                    default_value: Some(transformed_default_value.replace_or_else(|| value)),
                });
            }
        }

        return TransformedValue::Keep;
    }

    fn transform_list<I, F, R>(&mut self, list: &[I], f: F) -> TransformedValue<Vec<I>>
    where
        I: Clone,
        F: Fn(&mut Self, &I) -> R,
        R: Into<Transformed<I>>,
    {
        let mut result = Vec::new();
        let mut has_changes = false;
        for (index, prev_item) in list.iter().enumerate() {
            let next_item: Transformed<_> = f(self, prev_item).into();
            match next_item {
                Transformed::Keep => {
                    if has_changes {
                        result.push(prev_item.clone());
                    }
                }
                Transformed::Replace(next_item) => {
                    if !has_changes {
                        debug_assert!(result.capacity() == 0);
                        result.reserve(list.len());
                        result.extend(list.iter().take(index).cloned());
                    }
                    result.push(next_item);
                    has_changes = true;
                }
            }
        }
        if has_changes {
            TransformedValue::Replace(result)
        } else {
            TransformedValue::Keep
        }
    }
}
