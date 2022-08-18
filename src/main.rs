mod app;
mod block;
mod crossterm;
mod data;
mod files;
mod history;
mod hits;
mod modes;
mod print;
mod tabs;
mod theme;
mod ui;
mod undo;

use crate::crossterm::run;
use clap::{arg, command, Command};
use std::{error::Error, time::Duration};

const ADD_FILE: &str = "add file to edit.";

fn main() -> Result<(), Box<dyn Error>> {
    let mut paths = Vec::new();
    let tick_rate = Duration::from_millis(1000);
    let matches = command!()
        .propagate_version(true)
        .subcommand_required(false)
        .arg_required_else_help(false)
        .subcommand(Command::new("add").about(ADD_FILE).arg(arg!([NAME])))
        .get_matches();

    match matches.subcommand() {
        Some(("add", sub_matches)) => {
            let r = sub_matches.get_one::<String>("NAME");
            if r.is_some() {
                paths.push(r.unwrap().clone());
            }
        }
        _ => {}
    }
    run(tick_rate, paths)?;
    Ok(())
}
