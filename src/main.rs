#![warn(clippy::unused_io_amount)]

use std::env::Args;
use std::process;

use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, BufRead, BufWriter, Write, self};

fn main() {
   let config = get_config(std::env::args()).unwrap_or_else(|err| {
       eprintln!("Error!: {}", err);
       process::exit(1);
   });

   run(config).unwrap_or_else(|err| {
       eprintln!("Error!: {}", err);
       process::exit(1);
   });
}

fn run(config: Config) -> Result<(), &'static str> {
    match config.command {
        Command::AddEvent(event) => {
            fs::copy("db", "db.tmp").unwrap();
            let temp_file = OpenOptions::new()
                    .append(true)
                    .open("db.tmp")
                    .unwrap();


            let mut writer = BufWriter::new(temp_file);
            writer.write(event.encode().as_bytes()).unwrap();
            writer.write("\n".as_bytes()).unwrap();
            drop(writer);

            fs::rename("db.tmp", "db").unwrap();

            println!("Added event `{}`", event.name);


        },
        Command::RemoveEvent(name) => {
            let file = File::open("db").unwrap();
            let reader = BufReader::new(file);

            let possible_events: Vec<(usize, String)> = reader.lines()
                                        .enumerate()
                                        .filter(|(_, x)| x.as_ref().unwrap().contains(&name))
                                        .map(|(i, x)| (i, Event::display(x.unwrap())))
                                        .collect::<Vec<(usize, String)>>();

            if possible_events.is_empty() { return Err("No Event Found that matches!")};

            let (rm_item, rm_line) = if possible_events.len() > 1 {
                possible_events.iter().enumerate().for_each(|(i, item)| println!("{}: {}", i + 1, item.1));
                println!("Please choose which event to delete (by number): ");
                let mut line = String::new();
                let mut line_int = 0;
                let input = io::stdin();

                while !(line_int > 0 && line_int <= possible_events.len()) {
                    input.read_line(&mut line).unwrap();
                    line_int = line.trim().parse::<usize>().unwrap();
                }

                (&possible_events[line_int - 1].1, possible_events[line_int - 1].0)
            } else {
                (&possible_events[0].1, possible_events[0].0)
            };

            let temp_file = File::create("db.tmp").unwrap();
            let mut writer = BufWriter::new(temp_file);
            let file = File::open("db").unwrap();
            let reader = BufReader::new(file);
            reader.lines()
                   .enumerate()
                   .for_each(|(i, x)| {
                       if i != rm_line {
                           writer.write(x.unwrap().as_bytes()).unwrap();
                           writer.write("\n".as_bytes()).unwrap();
                       }
                   });

            fs::rename("db.tmp", "db").unwrap();

            println!("Removed event `{}`", rm_item);
        }
    }

    Ok(())
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
        let name = args.next()
                        .filter(|x| !x.contains('|'))
                        .ok_or_else(|| "No/Invalid name argument given!")?;
        let description = args.next()
                        .filter(|x| !x.contains('|'))
                        .ok_or_else(|| "No/Invalid description argument given!")?;
        let date = args.next()
                        .filter(|x| !x.contains('|'))
                        .ok_or_else(|| "No/Invalid date argument given!")?;
        

        config.command = Command::AddEvent(Event {
            name,
            description,
            date,
        });
    } else if a == "remove" {
        let name = args.next().filter(|x| !x.contains('|'))
                        .ok_or_else(|| "No/Invalid name argument given!")?;

        config.command = Command::RemoveEvent(name);
    } else {
        return Err("No command given!")
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

impl Event {
    fn encode(&self) -> String {
        format!("{}|{}|{}", self.name, self.description, self.date)
    }

    fn decode(raw: String) -> Event {
        let items: Vec<&str> = raw.split('|').collect();
        Event {
            name: String::from(items[0]),
            description: String::from(items[1]),
            date: String::from(items[2]),
        }
    }

    fn display(raw: String) -> String {
        let event = Event::decode(raw);
        format!("{} - {}", event.name, event.date)
    }
}

