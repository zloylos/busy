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
    .subcommand(
      clap::Command::new("start")
        .arg(clap::Arg::new("project_name").required(true).index(1))
        .arg(clap::Arg::new("task_title").required(true).index(2))
        .arg(clap::Arg::new("tags").index(3).multiple_values(true)),
    )
    .subcommand(clap::Command::new("stop"))
    .subcommand(
      clap::Command::new("log")
        .arg(clap::Arg::new("full").long("full"))
        .arg(clap::Arg::new("days").default_value("-1").long("days"))
        .arg(clap::Arg::new("today").long("today")),
    )
    .subcommand(
      clap::Command::new("stat")
        .arg(clap::Arg::new("days").default_value("-1").long("days"))
        .arg(clap::Arg::new("today").long("today")),
    )
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
          viewer.log_task(&task, project_name, true);
        }
        Err(err) => println!("start task err: {}", err),
      };
    }

    Some("stop") => {
      match pomidorka.borrow_mut().stop() {
        Ok(task) => {
          println!("task stopped:");
          let project_id = task.project_id();
          viewer.log_task(
            &task,
            pomidorka.borrow().project_by_id(project_id).unwrap().name(),
            true,
          );
        }
        Err(err) => println!("couldn't stop: {}", err),
      };
    }

    Some("log") => {
      let subcommand_matches = matches.subcommand_matches("log").unwrap();
      let show_full = subcommand_matches.is_present("full");
      let show_today_only = subcommand_matches.is_present("today");

      let mut period = match show_today_only {
        true => chrono::Duration::seconds(
          chrono::Local::now()
            .time()
            .num_seconds_from_midnight()
            .into(),
        ),
        false => {
          chrono::Duration::days(chrono::Local::now().weekday().num_days_from_monday() as i64)
        }
      };
      let period_arg: i64 = subcommand_matches.value_of_t("days").unwrap();
      if period_arg != -1 {
        period = chrono::Duration::days(period_arg);
      }

      viewer.log_tasks_list(Some(period), show_full);
    }

    Some(subcmd) => println!("unknown subcommand {}", subcmd),
    None => println!("subcommand not found"),
  };
}
