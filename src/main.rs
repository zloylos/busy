extern crate chrono;
extern crate colored;
extern crate serde;
extern crate serde_json;

use std::{cell::RefCell, rc::Rc};

use chrono::{Datelike, Timelike};
use viewer::Viewer;

use crate::pomidorka::Pomidorka;

mod duration_fmt;
mod pomidorka;
mod project;
mod state;
mod storage;
mod task;
mod viewer;

fn main() {
  let matches = clap::Command::new("Pomidorka")
    .arg_required_else_help(true)
    .subcommand(clap::Command::new("start").args(&[
      clap::Arg::new("project_name").required(true).index(1),
      clap::Arg::new("task_title").required(true).index(2),
      clap::Arg::new("tags").index(3).multiple_values(true),
    ]))
    .subcommand(clap::Command::new("stop"))
    .subcommand(clap::Command::new("log").args(&[
      clap::Arg::new("full").long("full"),
      clap::Arg::new("days").long("days"),
      clap::Arg::new("today").long("today"),
    ]))
    .subcommand(clap::Command::new("stat").args(&[
      clap::Arg::new("days").long("days"),
      clap::Arg::new("today").long("today"),
    ]))
    .subcommand(clap::Command::new("projects"))
    .get_matches();

  let pomidorka = Rc::new(RefCell::new(Pomidorka::new()));
  let viewer = Viewer::new(Rc::clone(&pomidorka));

  match matches.subcommand_name() {
    Some("projects") => {
      viewer.print_projects();
    }

    Some("start") => {
      let command_matches = matches.subcommand_matches("start").unwrap();
      let project_name = command_matches.value_of("project_name").unwrap();
      let task_title = command_matches.value_of("task_title").unwrap();
      let tags: Vec<String> = command_matches
        .values_of_t("tags")
        .unwrap()
        .iter_mut()
        .map(|tag: &mut String| tag.strip_prefix("+").unwrap().to_string())
        .collect();

      match pomidorka.borrow_mut().start(project_name, task_title, tags) {
        Ok(task) => {
          println!("task started: ");
          viewer.log_task(&task, true);
        }
        Err(err) => println!("start task err: {}", err),
      };
    }

    Some("stop") => {
      match pomidorka.borrow_mut().stop() {
        Ok(task) => {
          println!("task stopped:");
          viewer.log_task(&task, true);
        }
        Err(err) => println!("couldn't stop: {}", err),
      };
    }

    Some("log") => {
      let subcommand_matches = matches.subcommand_matches("log").unwrap();
      let show_full = subcommand_matches.is_present("full");
      let show_today_only = subcommand_matches.is_present("today");

      let period_arg = subcommand_matches.value_of_t("days").ok();
      let period = get_period(period_arg, show_today_only);
      viewer.log_tasks_list(Some(period), show_full);
    }

    Some(subcmd) => println!("unknown subcommand {}", subcmd),
    None => println!("subcommand not found"),
  };
}

fn get_period(period_days: Option<i64>, show_today_only: bool) -> chrono::Duration {
  if period_days.is_some() {
    return chrono::Duration::days(period_days.unwrap());
  }

  return match show_today_only {
    true => chrono::Duration::seconds(
      chrono::Local::now()
        .time()
        .num_seconds_from_midnight()
        .into(),
    ),
    false => chrono::Duration::days(chrono::Local::now().weekday().num_days_from_monday() as i64),
  };
}
