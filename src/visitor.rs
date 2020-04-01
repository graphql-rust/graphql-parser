use std::slice;

use crate::common::Text;
use crate::query::{SelectionSet, Directive, Selection, Field};


pub trait Visit {
    fn visit<'x, D: 'x>(&'x self) -> <(&'x Self, &'x D) as VisitorData>::Data
        where (&'x Self, &'x D): VisitorData,
            <(&'x Self, &'x D) as VisitorData>::Data: CreateData<'x, &'x Self, &'x D>,
    {
        CreateData::new(self)
    }
}

impl<S> Visit for S { }

pub trait VisitorData {
    type Data;
}

#[derive(Debug)]
pub struct FieldIter<'a, T>
    where T: Text<'a>
{
    stack: Vec<slice::Iter<'a, Selection<'a, T>>>,
}

pub trait CreateData<'a, S: ?Sized, D: ?Sized> {
    fn new(v: S) -> Self;
}

impl<'a, T> CreateData<'a, &'a SelectionSet<'a, T>, &'a Field<'a, T>>
    for FieldIter<'a, T>
    where T: Text<'a>,
{
    fn new(v: &'a SelectionSet<'a, T>) -> Self {
        FieldIter {
            stack: vec![v.items.iter()],
        }
    }
}

impl<'a, T> VisitorData for (&'a SelectionSet<'a, T>, &'a Field<'a, T>)
    where T: Text<'a>,
{
    type Data = FieldIter<'a, T>;
}

impl<'a, T: 'a> Iterator for FieldIter<'a, T>
    where T: Text<'a>,
{
    type Item = &'a Field<'a, T>;
    fn next(&mut self) -> Option<&'a Field<'a, T>> {
        let ref mut stack = self.stack;
        while !stack.is_empty() {
            match stack.last_mut().and_then(|iter| iter.next()) {
                Some(Selection::Field(f)) => {
                    stack.push(f.selection_set.items.iter());
                    return Some(f);
                }
                Some(Selection::InlineFragment(f)) => {
                    stack.push(f.selection_set.items.iter());
                    continue;
                }
                Some(Selection::FragmentSpread(..)) => {}
                None => {
                    stack.pop();
                }
            }
        }
        return None;
    }
}

#[test]
fn test_field_iter() {
    use crate::parse_query;
    use crate::query::Definition::Operation;
    use crate::query::OperationDefinition::Query;

    let doc = parse_query::<&str>(r#"
        query TestQuery {
            users {
                id
                country {
                    id
                }
            }
        }
    "#).expect("Failed to parse query");
    let def = match doc.definitions.iter().next().unwrap() {
        Operation(Query(q)) => &q.selection_set,
        _ => unreachable!(),
    };
    let mut fields = 0;
    let mut field_names = Vec::new();
    for f in def.visit::<Field<_>>() {
        fields += 1;
        field_names.push(f.name);
    }
    assert_eq!(fields, 4);
    assert_eq!(field_names, vec!["users", "id", "country", "id"]);
}
