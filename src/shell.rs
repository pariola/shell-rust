use crate::{command, prelude::*};
use std::{
    env,
    io::{self, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

pub struct State {
    pub builtin_commands: Vec<String>,
}

#[derive(Default)]
pub struct Input {
    pub stdin: Option<Vec<u8>>,
    pub program: String,
    pub arguments: Vec<String>,
}

impl Input {
    pub fn new(cmd: &command::Command, stdin: Option<Vec<u8>>) -> Self {
        Self {
            stdin: stdin,
            program: cmd.program.clone(),
            arguments: cmd.arguments.to_vec(),
        }
    }
}

#[derive(Default)]
pub struct Output {
    pub exit: bool,
    pub code: i32,
    pub stdout: Option<Vec<u8>>,
}

impl Output {
    fn exit(code: i32) -> Self {
        Self {
            exit: true,
            code: code,
            ..Default::default()
        }
    }

    fn empty(code: i32) -> Self {
        Self {
            code: code,
            ..Default::default()
        }
    }

    fn with_output(code: i32, out: Vec<u8>) -> Self {
        Self {
            code: code,
            stdout: Some(out),
            ..Default::default()
        }
    }

    fn with_string_output(code: i32, out: &str) -> Self {
        Self {
            code: code,
            stdout: Some(out.as_bytes().to_vec()),
            ..Default::default()
        }
    }
}

pub type Runner = fn(&mut State, Input) -> Result<Output>;

pub fn cd(_: &mut State, input: Input) -> Result<Output> {
    let args = input.arguments;
    if args.len() != 1 {
        return Ok(Output::with_string_output(1, "cd: too many arguments\n"));
    }

    let full_path: PathBuf;

    let mut path = args[0].clone();
    if path.starts_with('~') {
        let home_dir = env::var("HOME").unwrap_or_default();
        path = path.replacen('~', &home_dir, 1);
        full_path = PathBuf::from(path);
    } else {
        full_path = env::current_dir().map_err(Error::IO)?.join(path);
    }

    if let Err(e) = env::set_current_dir(&full_path) {
        return match e.kind() {
            io::ErrorKind::NotFound => {
                let full_path_str = full_path.to_string_lossy();
                let msg = format!("cd: {full_path_str}: No such file or directory\n");
                Ok(Output::with_string_output(1, &msg))
            }
            _ => Err(Error::IO(e)),
        };
    }

    Ok(Output::empty(0))
}

pub fn exit(_: &mut State, _: Input) -> Result<Output> {
    Ok(Output::exit(0))
}

pub fn pwd(_: &mut State, _: Input) -> Result<Output> {
    let dir = env::current_dir().map_err(Error::IO)?;
    let msg = format!("{}\n", dir.to_string_lossy());
    Ok(Output::with_string_output(0, &msg))
}

pub fn echo(_: &mut State, input: Input) -> Result<Output> {
    let args = input.arguments;
    let mut msg = String::new();

    for (i, s) in args.iter().enumerate() {
        msg.push_str(s);

        // don't add white space after last element
        if i < args.len() - 1 {
            msg.push(' ');
        }
    }

    msg.push('\n');

    Ok(Output::with_string_output(0, &msg))
}

pub fn type_cmd(state: &mut State, input: Input) -> Result<Output> {
    let args = input.arguments;

    if args.len() != 1 {
        return Ok(Output::with_string_output(1, "type expects 1 argument\n"));
    }

    let command = &args[0];

    if state.builtin_commands.contains(&command) {
        let msg = format!("{command} is a shell builtin\n");
        return Ok(Output::with_string_output(0, &msg));
    }

    match find_in_path(&command) {
        Ok(None) => {
            let msg = format!("{command}: not found\n");
            Ok(Output::with_string_output(1, &msg))
        }
        Ok(Some(v)) => {
            let msg = format!("{command} is {v}\n");
            Ok(Output::with_string_output(0, &msg))
        }
        Err(e) => Err(e),
    }
}

pub fn external(_: &mut State, input: Input) -> Result<Output> {
    let program = &input.program;

    let location = match find_in_path(program)? {
        Some(location) => location,
        None => {
            let msg = format!("{program}: not found\n");
            return Ok(Output::with_string_output(127, &msg));
        }
    };

    let stdin = match input.stdin {
        None => Stdio::inherit(),
        Some(_) => Stdio::piped(),
    };

    let mut child = Command::new(location)
        .args(&input.arguments)
        .stdin(stdin) // pipe stdin
        .stdout(Stdio::piped()) // pipe stdout
        .stderr(Stdio::inherit()) // pipe stderr
        .spawn()
        .map_err(Error::IO)?; // executes child process synchronously

    // write input to child process
    if let Some(b) = input.stdin {
        child
            .stdin
            .take()
            .unwrap()
            .write_all(&b)
            .map_err(Error::IO)?;
    }

    let o = child.wait_with_output().map_err(Error::IO)?;

    let exit_code = o.status.code().unwrap();
    Ok(Output::with_output(exit_code, o.stdout))
}

fn find_in_path(program: &str) -> Result<Option<String>> {
    let raw_path = env::var("PATH").unwrap_or_default();

    for dir in raw_path.split(':') {
        let p = Path::new(dir).join(&program);
        if p.try_exists().map_err(Error::IO)? {
            return Ok(Some(p.to_string_lossy().into_owned()));
        }
    }

    Ok(None)
}
