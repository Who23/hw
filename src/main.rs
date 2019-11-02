use std::env::Args;
use std::fs;
use std::process;

fn main() {
   let config = get_config(std::env::args()).unwrap_or_else(|err| {
       eprintln!("Error!: {}", err);
       process::exit(1);
   });

   
}

fn run() {

}

fn get_config(mut args: Args) -> Result<Config, &'static str> {
    // to get the first item out of the way, which is jistt the name
    args.next();

    // temp config
    let mut config = Config { 
        command: Command::RemoveEvent(String::from("blah")),
    };
    
    // get command name
    let a = args.next().ok_or_else(|| "No command given!")?;

    // add command to config object
    if a == "add" {
        let name = args.next().ok_or_else(|| "No name argument given!")?;
        let description = args.next().ok_or_else(|| "No description argument given!")?;
        let date = args.next().ok_or_else(|| "No date argument given!")?;

        config.command = Command::AddEvent(Event {
            name,
            description,
            date,
        });
    } else if a == "remove" {
        let name = args.next().unwrap();

        config.command = Command::RemoveEvent(name);
    }

    // return config
    Ok(config)
}

#[derive(Debug)]
struct Config {
    command: Command,
}

#[derive(Debug)]
enum Command {
    AddEvent(Event),
    RemoveEvent(String),
}

#[derive(Debug)]
struct Event {
    name: String,
    description: String,
    date: String,
}

