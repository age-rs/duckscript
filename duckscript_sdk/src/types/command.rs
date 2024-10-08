use crate::types::scope::clear;
use crate::types::scope::set_line_context_name;
use crate::utils::eval;
use crate::utils::state::{get_handles_sub_state, put_handle};
use duckscript::parser;
use duckscript::types::command::{Command, CommandInvocationContext, CommandResult};
use duckscript::types::error::ScriptError;
use duckscript::types::instruction::Instruction;
use duckscript::types::runtime::StateValue;

#[derive(Clone)]
pub(crate) struct AliasCommand {
    name: String,
    aliases: Vec<String>,
    help: String,
    scope_name: String,
    raw_command: String,
    arguments_amount: usize,
    instructions: Vec<Instruction>,
}

impl AliasCommand {
    fn new(
        name: String,
        aliases: Vec<String>,
        help: String,
        scope_name: String,
        raw_command: String,
        arguments_amount: usize,
    ) -> Result<AliasCommand, ScriptError> {
        let instructions = parser::parse_text(&raw_command)?;

        let mut scope_name_with_prefix = "scope::".to_string();
        scope_name_with_prefix.push_str(&scope_name);

        Ok(AliasCommand {
            name,
            aliases,
            help,
            scope_name: scope_name_with_prefix,
            raw_command,
            arguments_amount,
            instructions,
        })
    }
}

impl Command for AliasCommand {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn aliases(&self) -> Vec<String> {
        self.aliases.clone()
    }

    fn help(&self) -> String {
        format!(
            r#"
{}

#### Source:
<details>
  <summary>Show Source</summary>

```sh
{}
```
</details>

"#,
            &self.help, &self.raw_command
        )
        .to_string()
    }

    fn clone_and_box(&self) -> Box<dyn Command> {
        Box::new((*self).clone())
    }

    fn run(&self, context: CommandInvocationContext) -> CommandResult {
        if context.arguments.len() < self.arguments_amount {
            CommandResult::Error("Invalid arguments provided.".to_string())
        } else {
            let start_count = context.variables.len();
            let line_context_name = set_line_context_name(&self.scope_name, context.state);

            // define script arguments
            let mut handle_option = None;
            if !context.arguments.is_empty() {
                let mut index = 0;
                let mut array = vec![];
                for argument in context.arguments {
                    index = index + 1;
                    let mut key = self.scope_name.clone();
                    key.push_str("::argument::");
                    key.push_str(&index.to_string());

                    context.variables.insert(key, argument.clone());

                    array.push(StateValue::String(argument.clone()));
                }

                let handle = put_handle(context.state, StateValue::List(array));

                let mut key = self.scope_name.clone();
                key.push_str("::arguments");

                context.variables.insert(key, handle.clone());
                handle_option = Some(handle);
            }

            let (flow_result, flow_output) = eval::eval_instructions(
                &self.instructions,
                context.commands,
                context.state,
                context.variables,
                context.env,
                0,
            );

            match handle_option {
                Some(handle) => {
                    let handle_state = get_handles_sub_state(context.state);
                    match handle_state.remove(&handle) {
                        _ => (),
                    }
                }
                None => (),
            }
            clear(&self.scope_name, context.variables);
            set_line_context_name(&line_context_name, context.state);

            let end_count = context.variables.len();
            if start_count < end_count {
                CommandResult::Crash(
                    format!(
                        "Memory leak detected, delta variables count: {}",
                        end_count - start_count
                    )
                    .to_string(),
                )
            } else {
                match flow_result {
                    Some(result) => result,
                    None => CommandResult::Continue(flow_output),
                }
            }
        }
    }
}

#[derive(Clone)]
struct DocOnlyCommand {
    name: String,
    help: String,
}

impl Command for DocOnlyCommand {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn help(&self) -> String {
        self.help.clone()
    }

    fn clone_and_box(&self) -> Box<dyn Command> {
        Box::new((*self).clone())
    }

    fn run(&self, _context: CommandInvocationContext) -> CommandResult {
        CommandResult::Error("Documentation only commands should not be executed.".to_string())
    }
}

pub(crate) fn create_alias_command(
    name: String,
    aliases: Vec<String>,
    help: String,
    scope_name: String,
    script: String,
    arguments_amount: usize,
) -> Result<AliasCommand, ScriptError> {
    let raw_command = script.to_string();

    let command = AliasCommand::new(
        name,
        aliases,
        help,
        scope_name,
        raw_command,
        arguments_amount,
    )?;

    Ok(command)
}

pub(crate) fn create_doc_only_command(name: &str, help: &str) -> Box<dyn Command> {
    Box::new(DocOnlyCommand {
        name: name.to_string(),
        help: help.to_string(),
    })
}
