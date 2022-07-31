extern crate chrono;
extern crate colored;
extern crate serde;
extern crate serde_json;

use std::{cell::RefCell, collections::HashSet, rc::Rc};

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
    .subcommand(
      clap::Command::new("log").args(&[
        clap::Arg::new("days").long("days").takes_value(true),
        clap::Arg::new("full").long("full"),
        clap::Arg::new("today").long("today"),
        clap::Arg::new("project")
          .long("project")
          .multiple_values(true)
          .takes_value(true),
        clap::Arg::new("tag")
          .long("tag")
          .multiple_values(true)
          .takes_value(true),
      ]),
    )
    .subcommand(
      clap::Command::new("stat").args(&[
        clap::Arg::new("days").long("days").takes_value(true),
        clap::Arg::new("today").long("today"),
        clap::Arg::new("with-tags").long("with-tags"),
        clap::Arg::new("project")
          .long("project")
          .multiple_values(true)
          .takes_value(true),
        clap::Arg::new("tag")
          .long("tag")
          .multiple_values(true)
          .takes_value(true),
      ]),
    )
    .subcommand(clap::Command::new("projects"))
    .subcommand(clap::Command::new("edit"))
    .get_matches();

  let pomidorka = Rc::new(RefCell::new(Pomidorka::new()));
  let viewer = Viewer::new(Rc::clone(&pomidorka));

  match matches.subcommand_name() {
    Some("projects") => {
      clear_screen();
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

      let task_res = {
        let mut p = pomidorka.borrow_mut();
        p.start(project_name, task_title, tags)
      };
      match task_res {
        Ok(task) => {
          println!("task started: ");
          viewer.log_task(&task, true);
        }
        Err(err) => println!("start task err: {}", err),
      };
    }

    Some("stop") => {
      let task_res = {
        let mut p = pomidorka.borrow_mut();
        p.stop()
      };
      match task_res {
        Ok(task) => {
          println!("task stopped:");
          viewer.log_task(&task, true);
        }
        Err(err) => println!("couldn't stop: {}", err),
      };
    }

    Some("log") => {
      clear_screen();
      let subcommand_matches = matches.subcommand_matches("log").unwrap();
      let show_full = subcommand_matches.is_present("full");
      let show_today_only = subcommand_matches.is_present("today");
      let project_names = subcommand_matches
        .values_of_t("project")
        .ok()
        .unwrap_or_default();
      let project_ids = projects_to_ids_set(Rc::clone(&pomidorka), project_names);
      let tags = subcommand_matches
        .values_of_t("tag")
        .ok()
        .unwrap_or_default();

      let period_arg = subcommand_matches.value_of_t("days").ok();
      let period = get_period(period_arg, show_today_only);
      viewer.log_tasks_list(period, project_ids, &tags, show_full);
    }

    Some("stat") => {
      clear_screen();
      let subcommand_matches = matches.subcommand_matches("stat").unwrap();
      let show_today_only = subcommand_matches.is_present("today");
      let with_tags = subcommand_matches.is_present("with-tags");
      let project_names = subcommand_matches
        .values_of_t("project")
        .ok()
        .unwrap_or_default();
      let project_ids = projects_to_ids_set(Rc::clone(&pomidorka), project_names);
      let tags = subcommand_matches
        .values_of_t("tag")
        .ok()
        .unwrap_or_default();

      let period_arg = subcommand_matches.value_of_t("days").ok();
      let period = get_period(period_arg, show_today_only);
      viewer.show_stat(period, project_ids, &tags, with_tags);
    }

    Some("edit") => {
      // TODO(zloylos): rewrite it to use tmp file for specific task and then replace it
      // TODO(zloylos): change hardcoded nvim, to an editor from $EDITOR / $VISUAL
      let filepath = pomidorka.borrow().tasks_db_filepath().to_string();
      subprocess::Exec::cmd("nvim").arg(filepath).join().unwrap();
    }

    Some(subcmd) => println!("unknown subcommand {}", subcmd),
    None => println!("subcommand not found"),
  };
}

fn clear_screen() {
  subprocess::Exec::cmd("clear")
    .join()
    .expect("clean cmd doesn't work");
}

fn projects_to_ids_set(
  pomidorka: Rc<RefCell<Pomidorka>>,
  project_names: Vec<String>,
) -> Option<HashSet<u128>> {
  let mut project_ids = HashSet::new();
  for project_name in project_names.iter() {
    let project = pomidorka.borrow().project_by_name(project_name);
    if project.is_some() {
      project_ids.insert(project.unwrap().id());
    }
  }
  if project_ids.is_empty() {
    return None;
  }
  return Some(project_ids);
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
