use std::{fs, path::PathBuf};

use anyhow::{bail, Context};
use full_moon::{
    ast::{Call, Expression, FunctionArgs, Index, Stmt, Suffix, Value, Var},
    tokenizer::TokenType,
};

pub fn resolve_index_path(
    thunk_path: &PathBuf,
    packages_path: &PathBuf,
) -> anyhow::Result<(String, PathBuf)> {
    let source = fs::read_to_string(thunk_path).context("Failed to read path to thunk")?;

    let ast = full_moon::parse(&source).context("Failed to parse AST of thunk")?;
    assert!(ast.nodes().last_stmt().is_some());

    if let Some(Stmt::LocalAssignment(assignment)) = ast.nodes().stmts().nth(1) {
        let require = assignment.expressions().iter().next().unwrap();
        if let Ok(path_components) = match_require(require) {
            let path_components = path_components.unwrap();

            let index_name: &str = path_components.iter().nth(1).unwrap();

            let mut index_path = packages_path.clone();
            index_path.push(format!("_Index/{index_name}"));

            if !index_path.exists() {
                bail!("Resolved _Index path does not exist")
            }

            return Ok((index_name.to_owned(), index_path));
        }
    }

    bail!("Failed to parse require in package thunk");
}

// https://github.com/JohnnyMorganz/wally-package-types/blob/master/src/command.rs#L50
fn expression_to_components(expression: &Expression) -> Vec<String> {
    let mut components = Vec::new();

    match expression {
        Expression::Value { value, .. } => match &**value {
            Value::Var(Var::Expression(var_expression)) => {
                components.push(var_expression.prefix().to_string().trim().to_string());

                for suffix in var_expression.suffixes() {
                    match suffix {
                        Suffix::Index(index) => match index {
                            Index::Dot { name, .. } => {
                                components.push(name.to_string().trim().to_string());
                            }
                            Index::Brackets { expression, .. } => match expression {
                                Expression::Value { value, .. } => match &**value {
                                    Value::String(name) => match name.token_type() {
                                        TokenType::StringLiteral { literal, .. } => {
                                            components.push(literal.trim().to_string());
                                        }
                                        _ => panic!("non-string brackets index"),
                                    },
                                    _ => panic!("non-string brackets index"),
                                },
                                _ => panic!("non-string brackets index"),
                            },
                            _ => panic!("unknown index"),
                        },
                        _ => panic!("incorrect suffix"),
                    }
                }
            }
            _ => panic!("unknown require expression"),
        },
        _ => panic!("unknown require expression"),
    };

    components
}

// https://github.com/JohnnyMorganz/wally-package-types/blob/master/src/command.rs#L90
fn match_require(expression: &Expression) -> anyhow::Result<Option<Vec<String>>> {
    match expression {
        Expression::Value { value, .. } => match &**value {
            Value::FunctionCall(call) => {
                if call.prefix().to_string().trim() == "require" && call.suffixes().count() == 1 {
                    if let Suffix::Call(Call::AnonymousCall(FunctionArgs::Parentheses {
                        arguments,
                        ..
                    })) = call.suffixes().next().unwrap()
                    {
                        if arguments.len() == 1 {
                            return Ok(Some(expression_to_components(
                                arguments.iter().next().unwrap(),
                            )));
                        }
                    }
                } else {
                    bail!("unknown require expression 3");
                }
            }
            _ => bail!("unknown require expression 2"),
        },
        _ => bail!("unknown require expression 1"),
    }

    Ok(None)
}
