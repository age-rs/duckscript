use crate::utils::pckg;
use duckscript::types::command::{Command, CommandArgs, CommandResult};

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;

cfg_if::cfg_if! {
    if #[cfg(windows)] {
        fn get_os_value() -> Result<String, String> {
            Ok("Windows".to_string())
        }
    } else {
        use uname::uname;

        fn get_os_value() -> Result<String, String> {
            match uname() {
                Ok(info) => Ok(info.sysname),
                Err(error) => Err(error.to_string()),
            }
        }
    }
}

#[derive(Clone)]
pub(crate) struct CommandImpl {
    package: String,
}

impl Command for CommandImpl {
    fn name(&self) -> String {
        pckg::concat(&self.package, "GetOSName")
    }

    fn aliases(&self) -> Vec<String> {
        vec!["os_name".to_string()]
    }

    fn help(&self) -> String {
        include_str!("help.md").to_string()
    }

    fn clone_and_box(&self) -> Box<dyn Command> {
        Box::new((*self).clone())
    }

    fn run(&self, _arguments: CommandArgs) -> CommandResult {
        match get_os_value() {
            Ok(value) => CommandResult::Continue(Some(value)),
            Err(error) => CommandResult::Error(error),
        }
    }
}

pub(crate) fn create(package: &str) -> Box<dyn Command> {
    Box::new(CommandImpl {
        package: package.to_string(),
    })
}
