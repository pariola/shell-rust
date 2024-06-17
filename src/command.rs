use std::{iter::Peekable, str::Chars};

#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct Variable(String, String);

#[derive(Debug, Default)]
pub struct Command {
    pub pipe: bool,
    pub program: String,
    pub arguments: Vec<String>,
    pub variables: Vec<Variable>,
}

pub fn parse(raw_input: String) -> Vec<Command> {
    let mut chars = raw_input.chars().peekable();

    let mut buf = String::with_capacity(raw_input.len() / 2);

    let mut commands: Vec<Command> = vec![];

    let mut cmd = Command::default();

    let mut should_pipe = false;

    while let Some(c) = chars.next() {
        match c {
            ' ' | '\n' | '|' if !buf.is_empty() && cmd.program.is_empty() => {
                cmd.pipe = should_pipe;
                cmd.program = buf.clone();
                cmd.arguments = parse_arguments(&mut chars);

                should_pipe = false; // reset pipe flag
                buf.clear(); // clear buf
                commands.push(cmd); // store command
                cmd = Command::default(); // reset command, set piped if necessary
            }

            // go forward
            ' ' | '\n' => continue,
            '|' => {
                should_pipe = true;
                continue;
            }

            '=' if cmd.program.is_empty() => {
                let var = parse_variable(&mut chars, buf.clone());
                cmd.variables.push(var);
                buf.clear();
            }

            c => buf.push(c),
        };
    }

    commands
}

fn parse_variable(chars: &mut Peekable<Chars>, name: String) -> Variable {
    Variable(name, parse_string(chars))
}

fn parse_arguments(chars: &mut Peekable<Chars>) -> Vec<String> {
    let mut args: Vec<String> = vec![];

    // continue as long as there are more characters to consume
    while let Some(c) = chars.peek() {
        match c {
            '|' | '\n' => break,

            ' ' => {
                chars.next(); // consume early
            }

            _ => args.push(parse_string(chars)),
        }
    }

    return args;
}

fn parse_string(chars: &mut Peekable<Chars>) -> String {
    let mut buf = String::new();
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

            c => buf.push(c),
        }

        // look forward
        match chars.peek() {
            None | Some(' ') | Some('\n') | Some('|') if open_quote.is_none() => break,
            _ => (),
        }
    }

    buf
}
