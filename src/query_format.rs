use std::fmt;
use format::{Displayable, Formatter, Style};
use query::*;

impl Document {
    /// Format a document according to style
    pub fn format(&self, style: &Style) -> String {
        let mut formatter = Formatter::new(style);
        self.display(&mut formatter);
        formatter.into_string()
    }
}

fn to_string<T: Displayable>(v: &T) -> String {
    let style = Style::default();
    let mut formatter = Formatter::new(&style);
    v.display(&mut formatter);
    formatter.into_string()
}

impl Displayable for Document {
    fn display(&self, f: &mut Formatter) {
        for item in &self.definitions {
            item.display(f);
        }
    }
}

impl Displayable for Definition {
    fn display(&self, f: &mut Formatter) {
        match *self {
            Definition::Operation(ref op) => op.display(f),
            Definition::Fragment(ref frag) => frag.display(f),
        }
    }
}

impl Displayable for OperationDefinition {
    fn display(&self, f: &mut Formatter) {
        match *self {
            OperationDefinition::SelectionSet(ref set) => set.display(f),
            OperationDefinition::Query(ref q) => q.display(f),
            OperationDefinition::Mutation(ref m) => m.display(f),
            OperationDefinition::Subscription(ref s) => s.display(f),
        }
    }
}

impl Displayable for FragmentDefinition {
    fn display(&self, f: &mut Formatter) {
        f.margin();
        f.indent();
        f.write("fragment ");
        f.write(&self.name);
        f.write(" ");
        self.type_condition.display(f);
        format_directives(&self.directives, f);
        f.write(" ");
        f.start_block();
        for item in &self.selection_set.items {
            item.display(f);
        }
        f.end_block();
    }
}

impl Displayable for SelectionSet {
    fn display(&self, f: &mut Formatter) {
        f.margin();
        f.indent();
        f.start_block();
        for item in &self.items {
            item.display(f);
        }
        f.end_block();
    }
}

impl Displayable for Selection {
    fn display(&self, f: &mut Formatter) {
        match *self {
            Selection::Field(ref fld) => fld.display(f),
            Selection::InlineFragment(ref frag) => frag.display(f),
            Selection::FragmentSpread(ref frag) => frag.display(f),
        }
    }
}

fn format_arguments(arguments: &[(String, Value)], f: &mut Formatter) {
    if arguments.len() > 0 {
        f.write("(");
        f.write(&arguments[0].0);
        f.write(": ");
        arguments[0].1.display(f);
        for arg in &arguments[1..] {
            f.write(", ");
            f.write(&arg.0);
            f.write(": ");
            arg.1.display(f);
        }
        f.write(")");
    }
}

impl Displayable for Field {
    fn display(&self, f: &mut Formatter) {
        f.indent();
        if let Some(ref alias) = self.alias {
            f.write(alias);
            f.write(": ");
        }
        f.write(&self.name);
        format_arguments(&self.arguments, f);
        format_directives(&self.directives, f);
        if self.selection_set.items.len() > 0 {
            f.write(" ");
            f.start_block();
            for item in &self.selection_set.items {
                item.display(f);
            }
            f.end_block();
        } else {
            f.endline();
        }
    }
}

impl Displayable for Query {
    fn display(&self, f: &mut Formatter) {
        f.margin();
        f.indent();
        f.write("query");
        if let Some(ref name) = self.name {
            f.write(" ");
            f.write(name);
            if self.variable_definitions.len() > 0 {
                f.write("(");
                self.variable_definitions[0].display(f);
                for var in &self.variable_definitions[1..] {
                    f.write(", ");
                    var.display(f);
                }
                f.write(")");
            }
        }
        format_directives(&self.directives, f);
        f.write(" ");
        f.start_block();
        for item in &self.selection_set.items {
            item.display(f);
        }
        f.end_block();
    }
}

impl Displayable for Mutation {
    fn display(&self, f: &mut Formatter) {
        f.margin();
        f.indent();
        f.write("mutation");
        if let Some(ref name) = self.name {
            f.write(" ");
            f.write(name);
            if self.variable_definitions.len() > 0 {
                f.write("(");
                for var in &self.variable_definitions {
                    var.display(f);
                }
                f.write(")");
            }
        }
        format_directives(&self.directives, f);
        f.write(" ");
        f.start_block();
        for item in &self.selection_set.items {
            item.display(f);
        }
        f.end_block();
    }
}

impl Displayable for Subscription {
    fn display(&self, f: &mut Formatter) {
        f.margin();
        f.indent();
        f.write("subscription");
        if let Some(ref name) = self.name {
            f.write(" ");
            f.write(name);
            if self.variable_definitions.len() > 0 {
                f.write("(");
                for var in &self.variable_definitions {
                    var.display(f);
                }
                f.write(")");
            }
        }
        format_directives(&self.directives, f);
        f.write(" ");
        f.start_block();
        for item in &self.selection_set.items {
            item.display(f);
        }
        f.end_block();
    }
}

impl Displayable for VariableDefinition {
    fn display(&self, f: &mut Formatter) {
        f.write("$");
        f.write(&self.name);
        f.write(": ");
        self.var_type.display(f);
        if let Some(ref default) = self.default_value {
            f.write(" = ");
            default.display(f);
        }
    }
}

impl Displayable for VariableType {
    fn display(&self, f: &mut Formatter) {
        match *self {
            VariableType::NamedType(ref name) => f.write(name),
            VariableType::ListType(ref typ) => {
                f.write("[");
                typ.display(f);
                f.write("]");
            }
            VariableType::NonNullType(ref typ) => {
                typ.display(f);
                f.write("!");
            }
        }
    }
}

impl Displayable for Value {
    fn display(&self, f: &mut Formatter) {
        match *self {
            Value::Variable(ref name) => {
                f.write("$");
                f.write(name);
            }
            Value::Int(ref num) => f.write(&format!("{}", num.0)),
            Value::Float(val) => f.write(&format!("{}", val)),
            Value::String(ref val) => f.write_quoted(val),
            Value::Boolean(true) => f.write("true"),
            Value::Boolean(false) => f.write("false"),
            Value::Null => f.write("null"),
            Value::Enum(ref name) => f.write(name),
            Value::List(ref items) => {
                f.write("[");
                if items.len() > 0 {
                    items[0].display(f);
                    for item in &items[1..] {
                        f.write(", ");
                        item.display(f);
                    }
                }
                f.write("]");
            }
            Value::Object(ref items) => {
                f.write("{");
                let mut first = true;
                for (ref name, ref value) in items.iter() {
                    if first {
                        first = false;
                    } else {
                        f.write(", ");
                    }
                    f.write(name);
                    f.write(": ");
                    value.display(f);
                }
                f.write("}");
            }
        }
    }
}

fn format_directives(dirs: &[Directive], f: &mut Formatter) {
    for dir in dirs {
        f.write(" ");
        dir.display(f);
    }
}

impl Displayable for InlineFragment {
    fn display(&self, f: &mut Formatter) {
        f.indent();
        f.write("...");
        if let Some(ref cond) = self.type_condition {
            f.write(" ");
            cond.display(f);
        }
        format_directives(&self.directives, f);
        f.write(" ");
        f.start_block();
        for item in &self.selection_set.items {
            item.display(f);
        }
        f.end_block();
    }
}

impl Displayable for TypeCondition {
    fn display(&self, f: &mut Formatter) {
        match *self {
            TypeCondition::On(ref name) => {
                f.write("on ");
                f.write(name);
            }
        }
    }
}

impl Displayable for FragmentSpread {
    fn display(&self, f: &mut Formatter) {
        f.indent();
        f.write("...");
        f.write(&self.fragment_name);
        format_directives(&self.directives, f);
        f.endline();
    }
}

impl Displayable for Directive {
    fn display(&self, f: &mut Formatter) {
        f.write("@");
        f.write(&self.name);
        format_arguments(&self.arguments, f);
    }
}

impl fmt::Display for Document {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&to_string(self))
    }
}
impl fmt::Display for Definition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&to_string(self))
    }
}
impl fmt::Display for OperationDefinition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&to_string(self))
    }
}
impl fmt::Display for FragmentDefinition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&to_string(self))
    }
}
impl fmt::Display for SelectionSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&to_string(self))
    }
}
impl fmt::Display for Selection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&to_string(self))
    }
}
impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&to_string(self))
    }
}
impl fmt::Display for Query {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&to_string(self))
    }
}
impl fmt::Display for Mutation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&to_string(self))
    }
}
impl fmt::Display for Subscription {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&to_string(self))
    }
}
impl fmt::Display for VariableDefinition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&to_string(self))
    }
}
impl fmt::Display for VariableType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&to_string(self))
    }
}
impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&to_string(self))
    }
}
impl fmt::Display for InlineFragment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&to_string(self))
    }
}
impl fmt::Display for TypeCondition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&to_string(self))
    }
}
impl fmt::Display for FragmentSpread {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&to_string(self))
    }
}
impl fmt::Display for Directive {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&to_string(self))
    }
}
