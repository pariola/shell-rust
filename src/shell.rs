use std::{
    env,
    io::{self, Error, Write},
    path::Path,
    process::{self, Command, Stdio},
};

pub struct State {
    pub home_dir: String,
    pub work_dir: String,
    pub builtin_commands: Vec<String>,
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

pub type RunResult = Result<Output, Error>;

pub type Runner = fn(&mut State, String, &[String], Option<Vec<u8>>) -> RunResult;

pub fn exit(_: &mut State, _: String, _: &[String], _: Option<Vec<u8>>) -> RunResult {
    Ok(Output::exit(0))
}

pub fn pwd(state: &mut State, _: String, _: &[String], _: Option<Vec<u8>>) -> RunResult {
    let msg = format!("{}\n", &state.work_dir);
    Ok(Output::with_string_output(0, &msg))
}

pub fn echo(_: &mut State, _: String, args: &[String], _: Option<Vec<u8>>) -> RunResult {
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

pub fn change_directory(
    state: &mut State,
    _: String,
    args: &[String],
    _: Option<Vec<u8>>,
) -> RunResult {
    if args.len() != 1 {
        return Ok(Output::with_string_output(1, "cd: too many arguments\n"));
    }

    let mut path = args[0].clone();
    if path.starts_with('~') {
        path = path.replacen('~', &state.home_dir, 1);
    }

    let full_path = Path::new(&state.work_dir).join(path);

    match full_path.canonicalize() {
        Ok(v) => {
            state.work_dir = v.to_string_lossy().into_owned();
            Ok(Output::empty(0))
        }

        Err(e) => match e.kind() {
            io::ErrorKind::NotFound => {
                let full_path_str = full_path.to_string_lossy();
                let msg = format!("cd: {full_path_str}: No such file or directory\n");
                Ok(Output::with_string_output(1, &msg))
            }

            _ => Err(e),
        },
    }
}

pub fn builtin_check(
    state: &mut State,
    _: String,
    args: &[String],
    _: Option<Vec<u8>>,
) -> RunResult {
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

fn find_in_path(program: &str) -> Result<Option<String>, io::Error> {
    let raw_path = env::var("PATH").unwrap_or_default();

    for dir in raw_path.split(':') {
        let p = Path::new(dir).join(&program);
        if p.try_exists()? {
            return Ok(Some(p.to_string_lossy().into_owned()));
        }
    }

    Ok(None)
}

#[allow(unused_variables)]
pub fn run_external(
    state: &mut State,
    program: String,
    args: &[String],
    input: Option<Vec<u8>>,
) -> RunResult {
    let location = match find_in_path(&program)? {
        Some(location) => location,
        None => {
            let msg = format!("{program}: not found\n");
            return Ok(Output::with_string_output(127, &msg));
        }
    };

    let mut child = Command::new(location)
        .args(args)
        .stdin(Stdio::piped()) // pipe stdin
        .stdout(Stdio::piped()) // pipe stdout
        .stderr(Stdio::inherit()) // pipe stderr
        .spawn()?; // executes child process synchronously

    // write input to child process
    if let Some(b) = input {
        child.stdin.take().unwrap().write_all(&b)?;
    }

    let o: process::Output = child.wait_with_output()?;

    let exit_code = o.status.code().unwrap();
    Ok(Output::with_output(exit_code, o.stdout))
}
