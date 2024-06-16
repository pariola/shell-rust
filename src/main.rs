#[allow(unused_imports)]
use std::io::{self, Error, Write};
use std::process::{Command, Stdio};
use std::{env, fs};

mod command;

static COMMANDS: &'static [&str] = &["exit", "echo", "type", "pwd", "cd"];

fn main() {
    let stdin = io::stdin();

    let mut work_dir: String = match env::current_dir() {
        Ok(v) => v.to_string_lossy().into_owned(),
        Err(e) => panic!("err: fetch current dir: {e}"),
    };

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();

        let cmd = command::parse(input);

        match cmd.program.as_str() {
            "" => (),
            "pwd" => println!("{work_dir}"),
            "cd" => change_directory(&mut work_dir, &cmd.arguments),
            "exit" => return,
            "echo" => echo(&cmd.arguments),
            "type" => type_cmd(&cmd.arguments),

            command => {
                let location = match find_in_path(command) {
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

fn echo(args: &[String]) {
    let mut msg = String::new();

    for (i, s) in args.iter().enumerate() {
        msg.push_str(s);

        // don't add white space after last element
        if i < args.len() - 1 {
            msg.push(' ');
        }
    }

    println!("{}", msg)
}

fn type_cmd(args: &[String]) {
    if args.len() != 1 {
        return println!("type expects 1 argument");
    }

    let command = args[0].as_str();
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
                return Ok(Some(entry.path().to_string_lossy().into_owned()));
            }
        }
    }

    Ok(None)
}

fn change_directory(work_dir: &mut String, args: &[String]) {
    if args.len() != 1 {
        return println!("cd expects 1 argument");
    }

    let path = &args[0];

    match fs::read_dir(path) {
        Ok(_) => *work_dir = path.clone(),

        Err(e) => match e.kind() {
            io::ErrorKind::NotFound => println!("cd: {}: No such file or directory", path),
            _ => println!("cd err: {e}"),
        },
    }
}
