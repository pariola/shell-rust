#[allow(unused_imports)]
use std::io::{self, Error, Write};
use std::{env, fs};

static COMMANDS: &'static [&str] = &["exit", "echo", "type"];

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

        let stream = &tokens[1..];

        match tokens[0] {
            "exit" => return,
            "echo" => echo(stream),
            "type" => type_cmd(stream),
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

fn type_cmd(stream: &[&str]) {
    if stream.len() != 1 {
        return println!("type expects 1 argument");
    }

    let command = stream[0];
    if COMMANDS.contains(&command) {
        return println!("{command} is a shell builtin");
    }

    let path = env::var("PATH").unwrap();

    let res = find_in_path(path.as_str(), command);
    match res {
        Result::Ok(None) => println!("{command}: not found"),
        Result::Ok(Some(answer)) => println!("{command} is {answer}"),
        Err(e) => println!("err: {}", e),
    }
}

fn find_in_path(path: &str, file_name: &str) -> Result<Option<String>, io::Error> {
    for dir in path.split(':') {
        let entries_result = fs::read_dir(dir);
        match entries_result {
            core::result::Result::Err(e) => match e.kind() {
                io::ErrorKind::NotFound => continue, // skip invalid dir
                _ => return Err(e),
            },
            _ => (),
        }

        for entry in entries_result.unwrap() {
            let entry = entry?;
            if entry.file_name() == file_name {
                return Ok(Some(format!("{}", entry.path().display())));
            }
        }
    }

    Ok(None)
}
