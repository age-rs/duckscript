use std::io::Write;
use std::process::{Child, Command, Stdio};

#[cfg(test)]
#[path = "./exec_test.rs"]
mod exec_test;

#[derive(Debug, Clone)]
pub(crate) enum ExecInput {
    None,
    External,
    Text(String),
}

pub(crate) fn exec(
    arguments: &Vec<String>,
    print_output: bool,
    input: ExecInput,
    start_index: usize,
) -> Result<(String, String, i32), String> {
    let child = spawn(arguments, print_output, false, input, start_index)?;

    match child.wait_with_output() {
        Ok(ref output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
            let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
            let exit_code = match output.status.code() {
                Some(value) => value,
                None => {
                    return Err(format!(
                        "Unable to extract exit code for command: {}",
                        &arguments[0]
                    )
                    .to_string());
                }
            };

            Ok((stdout, stderr, exit_code))
        }
        Err(error) => Err(error.to_string()),
    }
}

pub(crate) fn spawn(
    arguments: &Vec<String>,
    print_output: bool,
    output_blocking: bool,
    input: ExecInput,
    start_index: usize,
) -> Result<Child, String> {
    let mut command = create_command(
        arguments,
        print_output,
        output_blocking,
        &input,
        start_index,
    )?;

    match command.spawn() {
        Ok(mut child) => match input {
            ExecInput::Text(value) => match child.stdin.as_mut() {
                Some(stdin) => match stdin.write_all(value.as_bytes()) {
                    Ok(_) => Ok(child),
                    Err(error) => Err(error.to_string()),
                },
                None => Err("Unable to write input to process".to_string()),
            },
            _ => Ok(child),
        },
        Err(error) => Err(error.to_string()),
    }
}

fn create_command(
    arguments: &Vec<String>,
    print_output: bool,
    output_blocking: bool,
    input: &ExecInput,
    start_index: usize,
) -> Result<Command, String> {
    if arguments.len() <= start_index {
        Err("Command not provided.".to_string())
    } else {
        let mut command = Command::new(&arguments[start_index]);
        let argument_index = start_index + 1;

        for argument in &arguments[argument_index..] {
            command.arg(argument);
        }

        match input {
            ExecInput::None => command.stdin(Stdio::null()),
            ExecInput::External => command.stdin(Stdio::inherit()),
            ExecInput::Text(_) => command.stdin(Stdio::piped()),
        };

        if print_output {
            command.stdout(Stdio::inherit()).stderr(Stdio::inherit());
        } else if output_blocking {
            command.stdout(Stdio::null()).stderr(Stdio::null());
        } else {
            command.stdout(Stdio::piped()).stderr(Stdio::piped());
        }

        Ok(command)
    }
}
