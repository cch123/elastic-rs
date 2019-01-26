# Overview

Convert Bool Expression to Elasticsearch DSL.

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

paren_bool = { "(" ~ (expr) ~  ")" }

comp_expr = { field ~ op ~ value }

field = @{ (ASCII_ALPHA ~ ASCII_ALPHANUMERIC*) }
op = { "="| "!="| "<>"| "in" | "not in" | ">" | ">=" | "<" | "<=" | "like" }

value = { string_literal | num_literal }

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
