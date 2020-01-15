use battery::units::ratio::percent;
use battery::{Manager, Result, State};
use std::process::Command;
use std::thread;
use std::time;

use clap::{App, Arg};

fn is_number(v: String) -> std::result::Result<(), String> {
    match v.parse::<i32>() {
        Ok(_) => Ok(()),
        Err(_) => Err(String::from("Value is not a number")),
    }
}

macro_rules! debug_output {
    ($x:expr, $( $e:expr ),*) => {
        if $x {
            println!($($e, )*);
        }
    }
}

fn main() -> Result<()> {
    let app = App::new("battery-indicator")
        .version("0.1.0")
        .arg(
            Arg::with_name("percentage")
                .short("p")
                .long("percentage")
                .help("Specifies the percentage of battery at which to execute the given command")
                .takes_value(true)
                .default_value("20")
                .validator(is_number),
        )
        .arg(
            Arg::with_name("command")
                .help("Command to execute when running short on battery")
                .multiple(true)
                .required(true)
        )
        .arg(
            Arg::with_name("polling period")
                .short("n")
                .long("polling-period")
                .help("Specifies the amount of time between battery checks")
                .takes_value(true)
                .validator(is_number)
                .default_value("30"),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .help("Enables verbose output of the program")
        );
    let matches = app.get_matches();
    let percentage = {
        matches
            .value_of("percentage")
            .unwrap()
            .parse::<f32>()
            .unwrap_or(20.0)
    };
    let is_verbose = matches.is_present("verbose");

    let cmd_to_exec = {
        matches.values_of("command").unwrap().collect::<Vec<_>>().join(" ")
    };
    let mut is_notified = false;
    let time_between_checks = {
        time::Duration::from_secs_f32(
            matches
                .value_of("polling period")
                .unwrap()
                .parse::<f32>()
                .unwrap(),
        )
    };
    loop {
        let manager = Manager::new()?;
        for bat in manager.batteries()? {
            let bat = bat?;
            debug_output!(is_verbose, "{:?}", bat);
            match bat.state() {
                State::Discharging => {
                    if !is_notified {
                        let charge = bat.state_of_charge().get::<percent>() as f32;
                        debug_output!(is_verbose, "{}", charge);
                        if charge < percentage {
                            debug_output!(is_verbose, "Executing command: {}", &cmd_to_exec);
                            Command::new("/bin/sh")
                                .arg("-c")
                                .arg(&cmd_to_exec)
                                .status()
                                .expect("Failed to execute command");
                            is_notified = true;
                        }
                    }
                }
                State::Charging => {
                    is_notified = false;
                }
                _ => (),
            };
        }
        thread::sleep(time_between_checks);
    }
}
