# Overview

Convert Bool Expression to Elasticsearch DSL.

```
                                                    ┌─────────────────────────────────────────────┐
                                                    │{                                            │
                                                    │    "bool": {                                │
                                                    │        "should": [{                         │
                                                    │            "match": {                       │
                                                    │                "a": {                       │
                                                    │                    "query": "1",            │
                                                    │                    "type": "phrase"         │
                                                    │                }                            │
                                                    │            }                                │
                                                    │        }, {                                 │
                                                    │            "bool": {                        │
                                                    │                "must": [{                   │
                                                    │                    "match": {               │
                                                    │                        "b": {               │
┌─────────────────────────────┐                     │                            "query": "2",    │
│ a = 1 or (b = 2 and c = 3)  │────────────────────▶│                            "type": "phrase" │
└─────────────────────────────┘                     │                        }                    │
                                                    │                    }                        │
                                                    │                }, {                         │
                                                    │                    "match": {               │
                                                    │                        "c": {               │
                                                    │                            "query": "3",    │
                                                    │                            "type": "phrase" │
                                                    │                        }                    │
                                                    │                    }                        │
                                                    │                }]                           │
                                                    │            }                                │
                                                    │        }]                                   │
                                                    │    }                                        │
                                                    │}                                            │
                                                    │                                             │
                                                    └─────────────────────────────────────────────┘
```

example :

```rust
fn main() {
    println!("hello world");
}
```

grammar :

```peg
bool_expr = { SOI ~ expr ~ EOI }

expr = {
    and_expr
    | or_expr
    | paren_bool
    | comp_expr
}

and_expr = {
    (paren_bool | comp_expr) ~ "and" ~ (expr)
}

or_expr = {
    (paren_bool | comp_expr)  ~ "or" ~ (expr)
}

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

