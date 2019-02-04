# Overview

Convert Bool Expression to Elasticsearch DSL.

```
                                          +----------------------------------------------------+
                                          |{                                                   |
                                          |    "query": {                                      |
                                          |        "bool": {                                   |
                                          |            "must": [{                              |
                                          |                "match": {                          |
                                          |                    "a": {                          |
                                          |                        "query": "1",               |
                                          |                        "type": "phrase"            |
                                          |                    }                               |
                                          |                }                                   |
                                          |            }, {                                    |
                                          |                "bool": {                           |
                                          |                    "must": [{                      |
                                          |                        "match": {                  |
                                          |                            "b": {                  |
+-----------------------------+           |                                "query": "2",       |
|a = 1 and (b = 2 and (c = 3))|---------->|                                "type": "phrase"    |
+-----------------------------+           |                            }                       |
                                          |                        }                           |
                                          |                    }, {                            |
                                          |                        "match": {                  |
                                          |                            "c": {                  |
                                          |                                "query": "3",       |
                                          |                                "type": "phrase"    |
                                          |                            }                       |
                                          |                        }                           |
                                          |                    }]                              |
                                          |                }                                   |
                                          |            }]                                      |
                                          |        }                                           |
                                          |    }                                               |
                                          |}                                                   |
                                          +----------------------------------------------------+
```

Example:

Add:

```toml
[dependencies]
elastic_query = "0.4.4"
```

To your `Cargo.toml`, then use as follows:

```rust
extern crate elastic_query;

fn main() {
    let result = elastic_query::convert("a = 1 and b in (1,2,3)".to_string(), 0, 100, vec![], vec![]).unwrap();
    println!("{}", result);
}

```

Grammar:

```peg
bool_expr = { SOI ~ expr ~ EOI }

expr = {
    (paren_bool | comp_expr) ~ ( (and_op|or_op)~ (paren_bool| comp_expr))*
}

and_op = { "and" }
or_op = { "or" }

paren_bool = { "(" ~ expr ~  ")" }

comp_expr = { field ~ op ~ value }

field = @{ (ASCII_ALPHA ~ ASCII_ALPHANUMERIC*) }
op = { eq | neq | op_in | op_not_in | gt | gte | lt | lte | like | not_like }
eq = { "=" }
neq = { "!=" | "<>"}
op_in = { "in" }
op_not_in= { "not" ~ "in"}
gt = { ">" }
gte = { ">=" }
lt = { "<" }
lte = { "<=" }
like = { "like" }
not_like = { "not" ~ "like" }

value = {
    string_literal
    | num_literal
    | "(" ~ string_literal ~("," ~ string_literal)* ~ ")"
    | "(" ~ num_literal ~("," ~ num_literal)* ~ ")"
}

num_literal = @{
    "-"?
    ~ ("0" | ASCII_NONZERO_DIGIT ~ ASCII_DIGIT*)
    ~ ("." ~ ASCII_DIGIT*)?
    ~ (^"e" ~ ("+" | "-")? ~ ASCII_DIGIT+)?
}

string_literal = ${ "\"" ~ string ~ "\"" }
string = @{ char* }
char = {
    !("\"" | "\\") ~ ANY
    | "\\" ~ ("\"" | "\\" | "/" | "b" | "f" | "n" | "r" | "t")
    | "\\" ~ ("u" ~ ASCII_HEX_DIGIT{4})
}

WHITESPACE = _{ " " | "\n" | "\r" }
```

