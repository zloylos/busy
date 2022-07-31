extern crate chrono;
extern crate colored;
extern crate serde;
extern crate serde_json;

use chrono::{Datelike, Timelike};
use colored::*;

use crate::{
  duration_fmt::{format_duration, format_duration_without_paddings},
  pomidorka::Pomidorka,
};

mod duration_fmt;
mod pomidorka;
mod project;
mod state;
mod storage;
mod task;

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
    .subcommand(clap::Command::new("projects"))
    .get_matches();

  let mut pomidorka = Pomidorka::new();
  match matches.subcommand_name() {
    Some("projects") => {
      print_projects(&pomidorka);
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

      match pomidorka.start(project_name, task_title, tags) {
        Ok(task) => {
          println!("task started: ");
          log_task(&task, project_name, true);
        }
        Err(err) => println!("start task err: {}", err),
      };
    }

    Some("stop") => {
      match pomidorka.stop() {
        Ok(task) => {
          println!("task stopped:");
          let project_id = task.project_id();
          log_task(
            &task,
            pomidorka.project_by_id(project_id).unwrap().name(),
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

      log_tasks_list(&pomidorka, Some(period), show_full);
    }

    Some(subcmd) => println!("unknown subcommand {}", subcmd),
    None => println!("subcommand not found"),
  };
}

fn log_tasks_list(pomidorka: &Pomidorka, period: Option<chrono::Duration>, full: bool) {
  let tasks = pomidorka.tasks(period.unwrap());
  if tasks.is_empty() {
    println!("no tasks to show");
    return;
  }

  println!("{}", "".clear());

  let mut by_dates: Vec<Vec<&task::Task>> = Vec::new();
  let mut date = None;
  for t in tasks.iter() {
    let task_date = t.start_time().date();
    if date.is_none() || date.unwrap() != task_date {
      by_dates.push(Vec::new());
      date = Some(task_date);
    }
    by_dates.last_mut().unwrap().push(t);
  }

  for tasks in by_dates.iter() {
    let date = tasks.first().unwrap().start_time().date();
    let total_time = tasks
      .iter()
      .map(|t| t.duration())
      .reduce(|acc, new_d| acc + new_d);

    println!(
      "{} — {}",
      date.format("%A, %d %B %Y").to_string().bold().cyan(),
      format_duration_without_paddings(total_time.unwrap())
        .bold()
        .bright_yellow()
    );

    for t in tasks.iter() {
      let mut project_name = "default".to_string();
      if let Some(task_project) = pomidorka.project_by_id(t.project_id()) {
        project_name = task_project.name().to_string();
      }
      log_task(t, project_name.as_str(), full);
    }
    println!("");
  }
}

fn log_task(task: &task::Task, project_name: &str, full: bool) {
  let stop_time = task.stop_time();
  let stop_time_msg = stop_time
    .unwrap_or(chrono::Local::now())
    .naive_local()
    .format("%H:%M")
    .to_string();

  let colored_stop_time_msg = match stop_time.is_some() {
    true => stop_time_msg.green(),
    false => stop_time_msg.yellow(),
  };

  let tags: Vec<String> = task
    .tags()
    .iter()
    .map(|tag| tag.cyan().to_string())
    .collect();
  let tags_str = tags.join(", ");

  println!(
    "{}",
    format!(
      "{padding}{task_id:04}  {start_time} to {stop_time} {duration:11}  {project:8}  [{tags}]",
      padding = " ".repeat(5),
      task_id = task.id(),
      start_time = task
        .start_time()
        .naive_local()
        .format("%H:%M")
        .to_string()
        .green(),
      stop_time = colored_stop_time_msg,
      duration = format_duration(task.duration()),
      project = project_name.red(),
      tags = tags_str.italic()
    )
  );
  if full {
    println!(
      "{}{}",
      " ".repeat(4 + 4 + 32),
      task.title().dimmed().italic()
    );
  }
}

fn print_projects(pomidorka: &Pomidorka) {
  for project in pomidorka.projects() {
    println!("{}: {}", project.id(), project.name());
  }
}
