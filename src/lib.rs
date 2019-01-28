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
    location: pest::error::InputLocation,
    expected: String,
}

pub fn convert(query: String, from: i32, size: i32) -> Result<serde_json::Value, ParseError> {
    let parse_result = ExprParser::parse(Rule::expr, query.as_str());
    match parse_result {
        Ok(mut expr_ast) => {
            let tree = simplify_ast(expr_ast.next().unwrap()).unwrap();
            let dsl = traverse(tree);
            Ok(json!({
               "query": dsl,
               "from" : from,
               "size" : size,
            }))
        }
        Err(err) => {
            // TODO: more friendly error
            Err(ParseError {
                location: err.location,
                expected: "".to_string(),
            })
        }
    }
}

use pest::iterators::Pair;

#[derive(Debug, Clone)]
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

fn traverse(n: Node) -> serde_json::Value {
    let walk_result = walk_tree(n.clone());
    match n {
        Node::CompExpr { .. } => return json!({
            "bool" : {
                "must" : [walk_result]
            }
        }),
        _ => return walk_result,
    }
}

fn walk_tree(n: Node) -> serde_json::Value {
    match n {
        Node::AndExpr { left, right } => {
            let left_val = walk_tree(*left);
            let right_val = walk_tree(*right);
            return serde_json::json!({
                "bool" : {
                    "must" : [left_val, right_val]
                }
            });
        }
        Node::OrExpr { left, right } => {
            let left_val = walk_tree(*left);
            let right_val = walk_tree(*right);

            return json!({
                "bool" : {
                    "should" : [left_val, right_val]
                }
            });
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
                });
            }
            ">=" => {
                return json!({"range" : {lhs : {"from" : rhs}}});
            }
            "<=" => {
                return json!({"range" : {lhs : {"to" : rhs}}});
            }
            ">" => return json!({"range" : {lhs : {"gt" : rhs}}}),
            "<" => return json!({"range" : {lhs : {"lt" : rhs}}}),
            "!=" | "<>" => {
                return json!({"bool" : {"must_not" : [{"match" : {lhs : {"query" : rhs, "type" : "phrase"}}}]}});
            }
            "in" => {
                let rhs = rhs.replace("\'", "\"");
                let r_vec: Vec<&str> = rhs
                    .trim_left_matches("(")
                    .trim_right_matches(")")
                    .split(",")
                    .collect();
                return json!({
                    "terms" : {lhs : r_vec}
                });
            }
            "not in" => {
                let rhs = rhs.replace("\'", "\"");
                let r_vec: Vec<&str> = rhs
                    .trim_left_matches("(")
                    .trim_right_matches(")")
                    .split(",")
                    .collect();
                return json!({"bool" : {"must_not" : {"terms" : { lhs : r_vec}}}});
            }
            _ => unreachable!(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::convert;
    use serde_json::json;

    struct TestCase {
        input: String,
        output: serde_json::Value,
    }

    #[test]
    fn test_convert() {
        let test_cases: Vec<TestCase> = vec![TestCase {
            input: "a=1".to_string(),
            output: json!({"query" : {"bool" : {"must" : [{"match" :{"a" : {"query" : "1", "type" : "phrase"}}}]}}, "from" : 1000, "size" : 1000}),
        }];
        test_cases.iter().for_each(|case| {
            assert_eq!(
                convert(case.input.clone(), 1000, 1000).unwrap(),
                case.output
            )
        });
    }
}
