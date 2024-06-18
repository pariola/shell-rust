mod command;
mod errors;
mod prelude;
mod shell;

use prelude::*;
use shell::{Input, Output};
use std::collections;
use std::io::{self, Write};

struct Shell {
    state: shell::State,
    runners: collections::HashMap<String, shell::Runner>,
}

fn main() -> Result<()> {
    let mut s = Shell {
        state: shell::State {
            builtin_commands: Vec::new(),
        },
        runners: collections::HashMap::new(),
    };

    s.register_command("cd", shell::cd);
    s.register_command("pwd", shell::pwd);
    s.register_command("echo", shell::echo);
    s.register_command("exit", shell::exit);
    s.register_command("type", shell::type_cmd);

    s.start()
}

impl Shell {
    fn register_command(&mut self, name: &str, runner: shell::Runner) {
        let name = name.to_string();
        self.runners.insert(name.clone(), runner);
        self.state.builtin_commands.push(name.clone());
    }

    fn run_command(
        &mut self,
        cmd: &command::Command,
        stdin: Option<Vec<u8>>,
    ) -> Result<shell::Output> {
        let input = Input::new(cmd, stdin);
        match self.runners.get(&cmd.program) {
            None => shell::external(&mut self.state, input),
            Some(runner) => runner(&mut self.state, input),
        }
    }

    fn start(&mut self) -> Result<()> {
        let stdin = io::stdin();

        loop {
            print!("$ ");
            io::stdout().flush().unwrap();

            let mut raw_input = String::new();
            stdin.read_line(&mut raw_input).unwrap();

            let mut last_output = Output::default();

            for cmd in command::parse(raw_input) {
                last_output = self.run_command(&cmd, last_output.stdout.clone())?;
            }

            match last_output.stdout {
                None => (),
                Some(out) => {
                    io::stdout().write_all(&out).unwrap();
                    io::stdout().flush().unwrap();
                }
            }

            if last_output.exit {
                std::process::exit(last_output.code)
            }
        }
    }
}
