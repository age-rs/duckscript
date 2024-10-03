use crate::utils::pckg;
use duckscript::types::command::{Command, CommandArgs, CommandResult};
use std::io::stdin;

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;

#[derive(Clone)]
pub(crate) struct CommandImpl {
    package: String,
}

impl Command for CommandImpl {
    fn name(&self) -> String {
        pckg::concat(&self.package, "ReadUserInput")
    }

    fn aliases(&self) -> Vec<String> {
        vec!["read".to_string()]
    }

    fn help(&self) -> String {
        include_str!("help.md").to_string()
    }

    fn clone_and_box(&self) -> Box<dyn Command> {
        Box::new((*self).clone())
    }

    fn run(&self, _arguments: CommandArgs) -> CommandResult {
        let mut text = String::new();

        let value = match stdin().read_line(&mut text) {
            Ok(_) => Some(text.trim().to_string()),
            Err(_) => None,
        };

        CommandResult::Continue(value)
    }
}

pub(crate) fn create(package: &str) -> Box<dyn Command> {
    Box::new(CommandImpl {
        package: package.to_string(),
    })
}
