#[allow(unused_imports)]
use std::io::{self, Error, Write};
use std::process::{Command, Stdio};
use std::{env, fs};

mod command;

static COMMANDS: &'static [&str] = &["exit", "echo", "type"];

fn main() {
    let stdin = io::stdin();

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();

        let cmd = command::parse(input);

        match cmd.program.as_str() {
            "" => (),
            "exit" => return,
            "echo" => echo(&cmd.arguments),
            "type" => type_cmd(&cmd.arguments),

            command => {
                let find_result = find_in_path(command);
                let location = match find_result {
                    Result::Ok(Some(location)) => location,

                    Result::Ok(None) => {
                        println!("{command}: not found");
                        continue;
                    }

                    Err(e) => {
                        println!("err: {}", e);
                        continue;
                    }
                };

                let run_result = Command::new(location)
                    .args(&cmd.arguments)
                    .stdin(Stdio::inherit()) // pipe stdin to parent's stdin
                    .stdout(Stdio::inherit()) // pipe stdout to parent's stdout
                    .stderr(Stdio::inherit()) // pipe stderr to parent's stderr
                    .output(); // executes the child process synchronously and captures its output.

                match run_result {
                    Ok(_) => (),
                    Err(e) => println!("err: {}", e),
                }
            }
        }
    }
}

fn echo(stream: &[String]) {
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

fn type_cmd(stream: &[String]) {
    if stream.len() != 1 {
        return println!("type expects 1 argument");
    }

    let command = stream[0].as_str();
    if COMMANDS.contains(&command) {
        return println!("{command} is a shell builtin");
    }

    let res = find_in_path(command);
    match res {
        Result::Ok(None) => println!("{command}: not found"),
        Result::Ok(Some(answer)) => println!("{command} is {answer}"),
        Err(e) => println!("err: {}", e),
    }
}

fn find_in_path(file_name: &str) -> Result<Option<String>, io::Error> {
    let path = match env::var("PATH") {
        core::result::Result::Ok(v) => v,
        core::result::Result::Err(_) => {
            return Ok(None); // return early if PATH is not set
        }
    };

    for dir in path.split(':') {
        let entries_result = fs::read_dir(dir);
        match entries_result {
            core::result::Result::Err(e) => {
                match e.kind() {
                    io::ErrorKind::NotFound => continue, // skip invalid dir
                    _ => return Err(e),
                };
            }
            _ => (), // continue
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
