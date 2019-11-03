#![warn(clippy::unused_io_amount)]

extern crate chrono;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

use std::cmp::Ordering;

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
        },
        Command::Info(name) => {
            let file = File::open("db").unwrap();
            let reader = BufReader::new(file);

            let possible_events: Vec<(usize, Event)> = reader.lines()
                                        .enumerate()
                                        .filter(|(_, x)| x.as_ref().unwrap().contains(&name))
                                        .map(|(i, x)| (i, Event::decode(x.unwrap())))
                                        .collect::<Vec<(usize, Event)>>();

            if possible_events.is_empty() { return Err("No Event Found that matches!")};

            let event = if possible_events.len() > 1 {
                possible_events.iter().enumerate().for_each(|(i, item)| println!("{}: {}", i + 1, item.1.int_display()));
                println!("Please choose which event to delete (by number): ");
                let mut line = String::new();
                let mut line_int = 0;
                let input = io::stdin();

                while !(line_int > 0 && line_int <= possible_events.len()) {
                    input.read_line(&mut line).unwrap();
                    line_int = line.trim().parse::<usize>().unwrap();
                }

                &possible_events[line_int - 1].1
            } else {
                &possible_events[0].1
            };

            println!("{}", event.int_display());
            if event.description == "" { println!("(No Description)"); } else { println!("{}", event.description); }

        }
        Command::DisplayEvents => {
            let db = File::open("db").unwrap();
            let reader = BufReader::new(db);
            let mut events: Vec<Event> = reader.lines()
                                           .map(|raw_event| Event::decode(raw_event.unwrap()))
                                           .collect();
            events.sort();
            let max_length = get_max_length(&events);
            let dashes = (0..max_length-6).map(|_| "─").collect::<String>();
            let spaces = (0..max_length+2).map(|_| " ").collect::<String>();

            println!("┌{}{}┐", events[0].date.format("%d/%m/%y"), dashes);
            println!("│{}│", spaces);
            let mut date = events[0].date.clone();
            for event in events {
                if event.date != date {
                    println!("│{}│", spaces);
                    println!("└────────{}┘", dashes);
                    println!("┌{}{}┐", event.date.format("%d/%m/%y"), dashes);
                    println!("│{}│", spaces);
                    date = event.date.clone();
                }

                let pre_indent = (max_length - event.name.len())/2;
                let post_indent = max_length - pre_indent - event.name.len();

                let pre_indent = (0..pre_indent).map(|_| " ").collect::<String>();
                let post_indent = (0..post_indent).map(|_| " ").collect::<String>();

                println!("│ {}{}{} │", pre_indent, event.name, post_indent);
            }
            println!("│{}│", spaces);
            println!("└────────{}┘", dashes);
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
        let date = args.next()
                        .filter(|x| !x.contains('|'))
                        .ok_or_else(|| "No/Invalid date argument given!")?;

        let date = match NaiveDate::parse_from_str(&date, "%d/%m/%y") {
            Ok(a) => a,
            Err(_) => return Err("Incorrect Date Formatting! (d/m/y)"),
        };

        let description = match args.next() {
            Some(des) => if !des.contains('|') { Ok(des) } else { Err("Invalid description given!") },
            None => Ok(String::from("")),
        }?;
        

        config.command = Command::AddEvent(Event {
            name,
            description,
            date,
        });
    } else if a == "remove" {
        let name = args.next().filter(|x| !x.contains('|'))
                        .ok_or_else(|| "No/Invalid name argument given!")?;

        config.command = Command::RemoveEvent(name);
    } else if a == "ls" {
        config.command = Command::DisplayEvents;
    } else if a == "info" {
        let name = args.next().filter(|x| !x.contains('|'))
                        .ok_or_else(|| "No/Invalid name argument given!")?;
        config.command = Command::Info(name);
    }

    // return config
    Ok(config)
}

fn get_max_length(arr: &Vec<Event>) -> usize {
    let mut max_length = 0;
    for el in arr {
        let el_length = el.name.len();
        if el_length > max_length { max_length = el_length };
    }
    if max_length < 15 {
        return 15;
    }
    max_length
}

#[derive(Debug)]
struct Config {
    command: Command,
}

#[derive(Debug)]
enum Command {
    AddEvent(Event),
    RemoveEvent(String),
    Info(String),
    DisplayEvents,
}

#[derive(Debug)]
struct Event {
    name: String,
    description: String,
    date: NaiveDate,
}

impl Event {
    fn encode(&self) -> String {
        format!("{}|{}|{}", self.name, self.description, self.date.format("%d/%m/%y"))
    }

    fn decode(raw: String) -> Event {
        let items: Vec<&str> = raw.split('|').collect();
        Event {
            name: String::from(items[0]),
            description: String::from(items[1]),
            date: NaiveDate::parse_from_str(items[2], "%d/%m/%y").unwrap(),
        }
    }

    fn display(raw: String) -> String {
        let event = Event::decode(raw);
        format!("{} - {}", event.name, event.date.format("%d/%m/%y"))
    }

    fn int_display(&self) -> String {
        format!("{} - {}", self.name, self.date.format("%d/%m/%y"))
    }
}

// implement traits needed for vector sort

impl Ord for Event {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_date_time = NaiveDateTime::new(self.date, NaiveTime::from_hms(0, 0, 0));
        let other_date_time = NaiveDateTime::new(other.date, NaiveTime::from_hms(0, 0, 0));

        self_date_time.timestamp().cmp(&other_date_time.timestamp())
    }
}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        let self_date_time = NaiveDateTime::new(self.date, NaiveTime::from_hms(0, 0, 0));
        let other_date_time = NaiveDateTime::new(other.date, NaiveTime::from_hms(0, 0, 0));

        self_date_time.timestamp() == other_date_time.timestamp()
    }
}

impl Eq for Event {}

