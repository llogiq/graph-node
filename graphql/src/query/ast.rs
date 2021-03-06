use graphql_parser::query::*;

use graph::prelude::QueryExecutionError;

/// Returns the operation for the given name (or the only operation if no name is defined).
pub fn get_operation<'a>(
    document: &'a Document,
    name: Option<&str>,
) -> Result<&'a OperationDefinition, QueryExecutionError> {
    let operations = get_operations(document);

    match (name, operations.len()) {
        (None, 1) => Ok(operations[0]),
        (None, _) => Err(QueryExecutionError::OperationNameRequired),
        (Some(s), n) if n > 0 => operations
            .into_iter()
            .find(|op| match get_operation_name(op) {
                Some(n) => s == n,
                None => false,
            }).ok_or(QueryExecutionError::OperationNotFound(s.to_string())),
        _ => Err(QueryExecutionError::OperationNameRequired),
    }
}

/// Returns all operation definitions in the document.
pub fn get_operations(document: &Document) -> Vec<&OperationDefinition> {
    document
        .definitions
        .iter()
        .map(|d| match d {
            Definition::Operation(op) => Some(op),
            _ => None,
        }).filter(|op| op.is_some())
        .map(|op| op.unwrap())
        .collect()
}

/// Returns the name of the given operation (if it has one).
pub fn get_operation_name(operation: &OperationDefinition) -> Option<&Name> {
    match operation {
        OperationDefinition::Mutation(m) => m.name.as_ref(),
        OperationDefinition::Query(q) => q.name.as_ref(),
        OperationDefinition::SelectionSet(_) => None,
        OperationDefinition::Subscription(s) => s.name.as_ref(),
    }
}

/// Looks up a directive in a selection, if it is provided.
pub fn get_directive(selection: &Selection, name: Name) -> Option<&Directive> {
    match selection {
        Selection::Field(field) => field
            .directives
            .iter()
            .find(|directive| directive.name == name),
        _ => None,
    }
}

/// Looks up the value of an argument in a vector of (name, value) tuples.
pub fn get_argument_value<'a>(arguments: &'a Vec<(Name, Value)>, name: &Name) -> Option<&'a Value> {
    arguments.iter().find(|(n, _)| n == name).map(|(_, v)| v)
}

/// Returns true if a selection should be skipped (as per the `@skip` directive).
pub fn skip_selection(selection: &Selection) -> bool {
    match get_directive(selection, "skip".to_string()) {
        Some(directive) => match get_argument_value(&directive.arguments, &"if".to_string()) {
            Some(val) => match val {
                Value::Boolean(skip_if) => *skip_if,
                _ => false,
            },
            None => true,
        },
        None => false,
    }
}

/// Returns true if a selection should be included (as per the `@include` directive).
pub fn include_selection(selection: &Selection) -> bool {
    match get_directive(selection, "include".to_string()) {
        Some(directive) => match get_argument_value(&directive.arguments, &"include".to_string()) {
            Some(val) => match val {
                Value::Boolean(include) => *include,
                _ => false,
            },
            None => true,
        },
        None => true,
    }
}

/// Returns the response key of a field, which is either its name or its alias (if there is one).
pub fn get_response_key(field: &Field) -> &Name {
    field.alias.as_ref().unwrap_or(&field.name)
}

/// Returns up the fragment with the given name, if it exists.
pub fn get_fragment<'a>(document: &'a Document, name: &Name) -> Option<&'a FragmentDefinition> {
    document
        .definitions
        .iter()
        .filter_map(|d| match d {
            Definition::Fragment(fd) => Some(fd),
            _ => None,
        }).find(|fd| &fd.name == name)
}
