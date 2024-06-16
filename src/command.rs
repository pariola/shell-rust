use std::{iter::Peekable, str::Chars};

#[allow(dead_code)]
pub struct Variable(String, String);

pub struct Command {
    pub program: String,
    pub arguments: Vec<String>,
    pub variables: Vec<Variable>,
}

pub fn parse(raw_input: String) -> Command {
    let mut chars = raw_input.chars().peekable();

    let mut buf = String::with_capacity(raw_input.len() / 2);

    let mut command = Command {
        program: String::new(),
        arguments: Vec::new(),
        variables: Vec::new(),
    };

    while let Some(c) = chars.next() {
        match c {
            ' ' | '\n' if !buf.is_empty() && command.program.is_empty() => {
                command.program = buf.clone();
                command.arguments = parse_arguments(&mut chars);

                // we should have consumed all characters
                assert_eq!(None, chars.peek());
                break;
            }

            ' ' | '\n' => continue, // go forward

            '=' if command.program.is_empty() => {
                let var = parse_variable(&mut chars, buf.clone());
                command.variables.push(var);
                buf.clear();
            }

            c => {
                // println!("writing '{}'", c);
                buf.push(c);
            }
        };
    }

    command
}

fn parse_variable(chars: &mut Peekable<Chars>, name: String) -> Variable {
    Variable(name, parse_string(chars))
}

fn parse_arguments(chars: &mut Peekable<Chars>) -> Vec<String> {
    let mut args: Vec<String> = vec![];

    // continue as long as there are more characters to consume
    while let Some(_) = chars.peek() {
        args.push(parse_string(chars));
    }

    return args;
}

fn parse_string(chars: &mut Peekable<Chars>) -> String {
    let mut buf = String::new();

    // check early
    match chars.peek() {
        None | Some(' ') | Some('\n') => return buf,
        _ => (),
    }

    let mut open_quote: Option<char> = None;

    while let Some(c) = chars.next() {
        match c {
            '\'' | '\"' => {
                if open_quote.is_none() {
                    open_quote = Some(c);
                    continue;
                }

                let last_quote = open_quote.unwrap();
                if c == last_quote {
                    open_quote = None; // remove open quote
                } else {
                    buf.push(c);
                }
            }

            ' ' | '\n' if open_quote.is_none() => break,

            c => buf.push(c),
        }
    }

    buf
}
