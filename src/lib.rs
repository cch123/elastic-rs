extern crate pest;
#[macro_use]
extern crate pest_derive;

extern crate serde_json;
use serde_json::json;

//use pest::error::Error;
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
            let dsl = walk_tree(expr_ast.next().unwrap(), true);
            //let dsl = traverse(tree);
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

fn walk_tree(record: Pair<Rule>, is_root: bool) -> serde_json::Value {
    match record.clone().as_rule() {
        Rule::bool_expr | Rule::expr | Rule::paren_bool => {
            return walk_tree(record.into_inner().next().unwrap(), is_root);
        }
        Rule::or_expr => {
            let mut iter = record.into_inner();
            let (left_val, right_val) = (
                walk_tree(iter.next().unwrap(), false),
                walk_tree(iter.next().unwrap(), false),
            );

            return serde_json::json!({
                "bool" : {
                    "should" : [left_val, right_val]
                }
            });
        }
        Rule::and_expr => {
            let mut iter = record.into_inner();
            let (left_val, right_val) = (
                walk_tree(iter.next().unwrap(), false),
                walk_tree(iter.next().unwrap(), false),
            );

            return serde_json::json!({
                "bool" : {
                    "must" : [left_val, right_val]
                }
            });
        }
        Rule::comp_expr => {
            let mut iter = record.into_inner();
            let (lhs, op, rhs) = (
                iter.next().unwrap().as_str().to_string(),
                iter.next().unwrap().into_inner().next().unwrap().as_rule(),
                iter.next().unwrap().as_str().to_string(),
            );

            let mut result: serde_json::Value;
            match op {
                Rule::eq | Rule::like => result = json!({"match" : {lhs : {"query" : rhs, "type" : "phrase" } } }),
                Rule::gte => result = json!({"range" : {lhs : {"from" : rhs}}}),
                Rule::lte => result = json!({"range" : {lhs : {"to" : rhs}}}),
                Rule::gt => result = json!({"range" : {lhs : {"gt" : rhs}}}),
                Rule::lt => result = json!({"range" : {lhs : {"lt" : rhs}}}),
                Rule::neq => result = json!({"bool" : {"must_not" : [{"match" : {lhs : {"query" : rhs, "type" : "phrase"}}}]}}),
                Rule::op_in => {
                    let rhs = rhs.replace("\'", "\"");
                    let r_vec: Vec<&str> = rhs
                        .trim_left_matches("(")
                        .trim_right_matches(")")
                        .split(",")
                        .map(|v| v.trim())
                        .collect();
                    result = json!({
                        "terms" : {lhs : r_vec}
                    });
                }
                Rule::op_not_in => {
                    let rhs = rhs.replace("\'", "\"");
                    let r_vec: Vec<&str> = rhs
                        .trim_left_matches("(")
                        .trim_right_matches(")")
                        .split(",")
                        .map(|v| v.trim())
                        .collect();
                    result = json!({"bool" : {"must_not" : {"terms" : { lhs : r_vec}}}});
                }

                _ => unreachable!(),
            }

            if is_root {
                return json!({
                    "bool" : {
                        "must" : [result]
                    }
                });
            }
            return result;
        }
        _ => unreachable!(),
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
        let test_cases: Vec<TestCase> = vec![
            TestCase {
                input: "a=1".to_string(),
                output: json!({"query" : {"bool" : {"must" : [{"match" :{"a" : {"query" : "1", "type" : "phrase"}}}]}}, "from" : 1000, "size" : 1000}),
            },
            TestCase {
                input: "a in (1,2,3)".to_string(),
                output: json!({"from":1000,"query":{"bool":{"must":[{"terms":{"a":["1","2","3"]}}]}},"size":1000}),
            },
            TestCase {
                input: "a in (   1, 2,  3)".to_string(),
                output: json!({"from":1000,"query":{"bool":{"must":[{"terms":{"a":["1","2","3"]}}]}},"size":1000}),
            },
        ];
        test_cases.iter().for_each(|case| {
            let output = convert(case.input.clone(), 1000, 1000).unwrap();
            println!("{}", &output);
            assert_eq!(output, case.output)
        });
    }
}
