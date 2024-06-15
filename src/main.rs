#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    let stdin = io::stdin();

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();

        let tokens = Vec::from_iter(input.split_whitespace());

        if tokens.len() < 1 {
            continue;
        }

        match tokens[0] {
            "exit" => return,
            "echo" => echo(&tokens[1..]),
            command => println!("{command}: command not found"),
        }
    }
}

fn echo(stream: &[&str]) {
    let mut msg = String::new();

    for (i, s) in stream.iter().enumerate() {
        msg.push_str(s);

        // don't add white space after last element
        if i < stream.len() - 1 {
            msg.push(' ');
        }
    }

    println!("{}", msg)
}
