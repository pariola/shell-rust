use std::io::{self, Error, Write};
use std::{collections, env};

mod command;
mod shell;

struct Shell {
    state: shell::State,
    commands: collections::HashMap<String, shell::Runner>,
}

fn main() {
    let mut s = Shell {
        state: shell::State {
            home_dir: get_home_dir(),
            work_dir: get_current_dir(),
            builtin_commands: Vec::new(),
        },
        commands: collections::HashMap::new(),
    };

    s.register_command("pwd", shell::pwd);
    s.register_command("echo", shell::echo);
    s.register_command("exit", shell::exit);
    s.register_command("type", shell::builtin_check);
    s.register_command("cd", shell::change_directory);

    s.start()
}

impl Shell {
    fn register_command(&mut self, name: &str, runner: shell::Runner) {
        let name = name.to_string();
        self.commands.insert(name.clone(), runner);
        self.state.builtin_commands.push(name.clone());
    }

    fn run_command(
        &mut self,
        cmd: &command::Command,
        input: Option<Vec<u8>>,
    ) -> Result<shell::Output, Error> {
        let program = cmd.program.clone();

        match self.commands.get(&cmd.program) {
            None => shell::run_external(&mut self.state, program, &cmd.arguments, input),
            Some(runner) => runner(&mut self.state, program, &cmd.arguments, input),
        }
    }

    fn start(&mut self) {
        let stdin = io::stdin();

        loop {
            print!("$ ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            stdin.read_line(&mut input).unwrap();

            let mut pipe: Option<Vec<u8>> = None;

            for cmd in command::parse(input) {
                match self.run_command(&cmd, pipe.clone()) {
                    Err(_) => (),
                    Ok(output) => {
                        if output.exit {
                            std::process::exit(output.code)
                        }

                        pipe = output.stdout.clone();
                    }
                }
            }

            match pipe {
                None => (),
                Some(pipe_input) => {
                    io::stdout().write_all(&pipe_input).unwrap();
                    io::stdout().flush().unwrap();
                }
            }
        }
    }
}

fn get_current_dir() -> String {
    match env::current_dir() {
        Ok(v) => v.to_string_lossy().into_owned(),
        Err(e) => panic!("err: fetch current dir: {e}"),
    }
}

fn get_home_dir() -> String {
    match env::var("HOME") {
        Ok(v) => v,
        Err(e) => panic!("env err: HOME not set: {e}"),
    }
}
