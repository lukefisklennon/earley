use std::{
    collections::{HashMap, HashSet},
    env::args,
    error::Error,
};

use abackus::{ParserBuilder, Tree};

const GRAMMAR: &str = r#"
    S    := NP Aux VP ;
    NP   := [ Det ] { AdjP } N { PP } | NP Conj NP ;
    PP   := P NP | PP Conj PP ;
    VP   := { AdvP } V [ NP ] [ AdjP ] [ CP ] { AdvP } { PP } { AdvP } | VP Conj VP ;
    CP   := C S | CP Conj CP ;
    AdvP := [ AdvP ] Adv | AdvP Conj AdvP ;
    AdjP := [ AdvP ] Adj [ PP ] | AdjP Conj AdjP ;
"#;

fn main() -> Result<(), Box<dyn Error>> {
    let input = args().skip(2).collect::<Vec<String>>().join(" ");
    let tagged_tokens = input.split_whitespace();
    let mut tokens = Vec::new();
    let mut tags = HashMap::new();

    for tagged_token in tagged_tokens {
        let mut split = tagged_token.split('.');
        let token = split.next().unwrap();
        tokens.push(token);

        if let Some(tag) = split.next() {
            tags.entry(pascal_case(tag))
                .or_insert(HashSet::new())
                .insert(token.to_string());
        }
    }

    let mut builder = ParserBuilder::default();

    for (tag, tokens) in tags {
        builder = builder.plug_terminal(tag, move |token| tokens.contains(&token.to_string()));
    }

    let start = args().nth(1).unwrap();
    let parse = builder.treeficator(GRAMMAR, start.as_str());

    for tree in parse(tokens.iter())? {
        println!("{}", s_expression(&tree).unwrap());
    }

    Ok(())
}

fn s_expression(tree: &Tree) -> Option<String> {
    match tree {
        Tree::Node(label, children) => {
            if label.starts_with('<') {
                match children.len() {
                    0 => return None,
                    _ => return s_expression(&children[0]),
                }
            }

            let expressions: Vec<String> = children.iter().filter_map(s_expression).collect();

            label
                .split_whitespace()
                .next()
                .map(|label| format!("[{} {}]", label, expressions.join(" ")))
        }
        Tree::Leaf(label, value) => {
            if label.starts_with('<') {
                return None;
            }

            Some(format!("[{} {}]", label, value.replace('_', " ")))
        }
    }
}

fn pascal_case(input: &str) -> String {
    let mut chars = input.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}
