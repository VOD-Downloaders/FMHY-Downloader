//use std;

/////////////////////////////////////////////////////
// CliOption
/////////////////////////////////////////////////////
#[derive(Debug, Clone, PartialEq)]
pub enum CliOption 
{
    Value(String),
    Flag(String),
    Option(String, String)
}

/////////////////////////////////////////////////////
// Parse functions
/////////////////////////////////////////////////////
// Types it can handle
// Value: value
// Flag: -flag
// Flag: --flag
// Option: --option=value
pub fn parse_cli_arguments(args: Vec<String>) -> Vec<CliOption> 
{
    let mut options: Vec<CliOption> = Vec::new();

    for i in 1..args.len()
    {
        let arg: &String = &args[i];

        // Option or Flag
        if arg.starts_with("--") 
        {
            // 2.. removes the creates a view starting after the first 2 characters (which removes the '--')
            let splits: Option<(&str, &str)> = arg[2..].split_once('=');

            if let Some((key, value)) = splits { // Option
                options.push(CliOption::Option(key.to_string(), value.to_string()));
            }
            else { // Flag
                options.push(CliOption::Flag(arg[2..].to_string()));
            }
        }
        // Flag
        else if arg.starts_with('-') {
            options.push(CliOption::Flag(arg[1..].to_string()));
        }
        // Value
        else {
            options.push(CliOption::Value(arg.clone()));
        }
    }

    return options;
}