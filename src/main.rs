use std::env;
use std::io::{stdin, stdout, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};

fn main() {
    loop {
        // need to explicitly flush this to ensure it prints before read_line
        let curr_dir;
        match env::current_dir() {
            Ok(res) => {
                curr_dir = res.as_path().display().to_string();
            }
            Err(_err) => {
                curr_dir = "/".to_string();
            }
        }
        print!("ferris: {} > ", curr_dir);
        match stdout().flush() {
            Ok(_) => {}
            Err(err) => {
                eprintln!("An error occured: {}", err);
            }
        }

        let mut input = String::new();
        match stdin().read_line(&mut input) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("An error occured: {}", err);
                continue;
            }
        }

        // must be peekable so we know when we are on the last command
        let mut commands = input.trim().split(" | ").peekable();
        let mut previous_command = None;

        while let Some(command) = commands.next() {
            // everything after the first whitespace character
            //     is interpreted as args to the command
            let mut parts = command.trim().split_whitespace();
            let command;
            match parts.next() {
                Some(res) => {
                    command = res;
                }
                None => {
                    continue;
                }
            }
            let args = parts;

            match command {
                "cd" => {
                    // default to '/' as new directory if one was not provided
                    let new_dir = args.peekable().peek().map_or("/", |x| *x);
                    let root = Path::new(new_dir);
                    if let Err(e) = env::set_current_dir(&root) {
                        eprintln!("{}", e);
                    }

                    previous_command = None;
                }
                "exit" => return,
                command => {
                    let stdin = previous_command.map_or(Stdio::inherit(), |output: Child| {
                        match output.stdout {
                            Some(res) => {
                                return Stdio::from(res);
                            }
                            None => {
                                panic!("Could not get stdio");
                            }
                        }
                    });

                    let stdout = if commands.peek().is_some() {
                        // there is another command piped behind this one
                        // prepare to send output to the next command
                        Stdio::piped()
                    } else {
                        // there are no more commands piped behind this one
                        // send output to shell stdout
                        Stdio::inherit()
                    };

                    let output = Command::new(command)
                        .args(args)
                        .stdin(stdin)
                        .stdout(stdout)
                        .spawn();

                    match output {
                        Ok(output) => {
                            previous_command = Some(output);
                        }
                        Err(err) => {
                            previous_command = None;
                            eprintln!("An error occured: {}", err);
                        }
                    };
                }
            }
        }

        if let Some(mut final_command) = previous_command {
            // block until the final command has finished
            match final_command.wait() {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("An error occured: {}", err);
                }
            }
        }
    }
}
