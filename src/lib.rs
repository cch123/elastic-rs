extern crate pest;
#[macro_use]
extern crate pest_derive;

extern crate serde_json;
use serde_json::json;

use pest::error::Error;
use pest::Parser;

#[derive(Parser)]
#[grammar = "expr.pest"]
pub struct ExprParser;

#[derive(Debug)]
pub struct ParseError {
    location: i32,
    expected: String,
    the_char: char,
}

use std::collections::HashMap;

pub fn convert_to_map() -> HashMap<String, String> {
    HashMap::new()
}

pub fn convert(input: String) -> Result<String, ParseError> {
    let parse_result = ExprParser::parse(Rule::expr, input.as_str());
    match parse_result {
        Ok(mut expr_ast) => {
            let tree = simplify_ast(expr_ast.next().unwrap()).unwrap();
            let dsl = traverse(tree);
            Ok(dsl)
        }
        Err(_) => {
            /*
            dbg!(err.variant);
            dbg!(err.location);
            dbg!(err.line_col);
            */
            Err(ParseError {
                location: 0,
                expected: "".to_string(),
                the_char: 'c',
            })
        }
    }
}

use pest::iterators::Pair;

#[derive(Debug)]
enum Node {
    AndExpr {
        left: Box<Node>,
        right: Box<Node>,
    },
    OrExpr {
        left: Box<Node>,
        right: Box<Node>,
    },
    CompExpr {
        lhs: String,
        op: String,
        rhs: String,
    },
}

fn simplify_ast(record: Pair<Rule>) -> Result<Node, Error<Rule>> {
    match record.clone().as_rule() {
        Rule::bool_expr | Rule::expr | Rule::paren_bool => {
            return simplify_ast(record.into_inner().next().unwrap());
        }
        Rule::or_expr => {
            let mut iter = record.into_inner();
            let (left_tree, right_tree) = (
                simplify_ast(iter.next().unwrap()).unwrap(),
                simplify_ast(iter.next().unwrap()).unwrap(),
            );
            return Ok(Node::OrExpr {
                left: Box::new(left_tree),
                right: Box::new(right_tree),
            });
        }
        Rule::and_expr => {
            let mut iter = record.into_inner();
            let (left_tree, right_tree) = (
                simplify_ast(iter.next().unwrap()).unwrap(),
                simplify_ast(iter.next().unwrap()).unwrap(),
            );
            return Ok(Node::AndExpr {
                left: Box::new(left_tree),
                right: Box::new(right_tree),
            });
        }
        Rule::comp_expr => {
            let mut iter = record.into_inner();
            let (field, op, value) = (
                iter.next().unwrap().as_str().to_string(),
                iter.next().unwrap().as_str().to_string(),
                iter.next().unwrap().as_str().to_string(),
            );

            return Ok(Node::CompExpr {
                lhs: field,
                op,
                rhs: value,
            });
        }
        _ => unreachable!(),
    }
}

fn traverse(n: Node) -> String {
    match n {
        Node::CompExpr { .. } => return format!(r#"{{"bool" : {{"must" : [{}]}}}}"#, walk_tree(n)),
        _ => return walk_tree(n),
    }
}

fn walk_tree2(n: Node) -> serde_json::Value {
    match n {
        Node::AndExpr { left, right } => {
            let left_str = walk_tree(*left);
            let right_str = walk_tree(*right);
            return serde_json::json!({
                "bool" : {
                    "must" : format!("[{}, {}]", left_str, right_str),
                }
            })
        }
        Node::OrExpr { left, right } => {
            let left_str = walk_tree(*left);
            let right_str = walk_tree(*right);

            return json!({
                "bool" : {
                    "should" : format!("[{}, {}]", left_str, right_str)
                }
            })
        }
        Node::CompExpr { lhs, op, rhs } => match op.as_str() {
            "=" | "like" => {
                return json!({
                    "match" : {
                        lhs : {
                            "query" : rhs,
                            "type" : "phrase"
                        }
                    }
                })
            }
            ">=" => {
                return json!({"range" : {lhs : {"from" : rhs}}});
            }
            "<=" => {
                return json!({"range" : {lhs : {"to" : rhs}}});
            }
            ">" => {
                return json!({"range" : {lhs : {"gt" : rhs}}})
            }
            "<" => {
                return json!({"range" : {lhs : {"lt" : rhs}}})
            }
            "!=" | "<>" => {
                return json!({"bool" : {"must_not" : [{"match" : {lhs : {"query" : rhs, "type" : "phrase"}}}]}});
            }
            "not in" => {
                return json!({"bool" : {"must_not" : [{"match" : {lhs : {"query" : rhs, "type" : "phrase"}}}]}});
                /*
                return format!(
                    r##"{{"bool" : {{"must_not" : {{"terms" : {{"{}" : {} }}}}}}}}"##,
                    lhs,
                    "[".to_string()
                        + rhs
                        .replace("\'", "\"")
                        .trim_left_matches("(")
                        .trim_right_matches(")")
                        + "]"
                );
                */
            }
            _ => unreachable!(),
        },
    }
}
fn walk_tree(n: Node) -> String {
    match n {
        Node::AndExpr { left, right } => {
            let left_str = walk_tree(*left);
            let right_str = walk_tree(*right);
            return format!(
                r##"{{"bool" : {{"must" : [{}, {}]}}}}"##,
                left_str, right_str
            );
        }
        Node::OrExpr { left, right } => {
            let left_str = walk_tree(*left);
            let right_str = walk_tree(*right);
            return format!(
                r##"{{"bool" : {{"should" : [{}, {}]}}}}"##,
                left_str, right_str
            );
        }
        Node::CompExpr { lhs, op, rhs } => match op.as_str() {
            "=" | "like" => {
                return format!(
                    r##"{{"match" : {{"{}" : {{"query" : "{}", "type" : "phrase"}}}}}}"##,
                    lhs, rhs
                );
            }
            ">=" => {
                return format!(r##"{{"range" : {{"{}" : {{"from" : "{}"}}}}}}""##, lhs, rhs);
            }
            "<=" => {
                return format!(r##"{{"range" : {{"{}" : {{"to" : "{}"}}}}}}""##, lhs, rhs);
            }
            ">" => {
                return format!(r##"{{"range" : {{"{}" : {{"gt" : "{}"}}}}}}""##, lhs, rhs);
            }
            "<" => {
                return format!(r##"{{"range" : {{"{}" : {{"lt" : "{}"}}}}}}""##, lhs, rhs);
            }
            "!=" | "<>" => {
                return format!(r##"{{"bool" : {{"must_not" : [{{"match" : {{"{}" : {{"query" : "{}", "type" : "phrase"}}}}}}]}}}}"##, lhs, rhs);
            }
            "in" => {
                return format!(
                    r##"{{"terms" : {{"{}" : {}}}}}"##,
                    lhs,
                    "[".to_string()
                        + rhs
                            .replace("\'", "\"")
                            .trim_left_matches("(")
                            .trim_right_matches(")")
                        + "]"
                );
            }
            "not in" => {
                return format!(
                    r##"{{"bool" : {{"must_not" : {{"terms" : {{"{}" : {} }}}}}}}}"##,
                    lhs,
                    "[".to_string()
                        + rhs
                            .replace("\'", "\"")
                            .trim_left_matches("(")
                            .trim_right_matches(")")
                        + "]"
                );
            }
            _ => unreachable!(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::convert;
    struct TestCase {
        input : String,
        output : String,
    }

    #[test]
    fn test_convert() {
        let test_cases:Vec<TestCase> = vec![];
        test_cases.iter().for_each(|case|{
            assert_eq!(convert(case.input.clone()).unwrap(), case.output)
        });
    }
}

