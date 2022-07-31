extern crate chrono;
extern crate colored;
extern crate serde;
extern crate serde_json;

use colored::*;

use crate::{duration_fmt::format_duration, pomidorka::Pomidorka};

mod duration_fmt;
mod pomidorka;
mod project;
mod state;
mod storage;
mod task;

fn main() {
  let matches = clap::App::new("Pomidorka")
    .subcommand(
      clap::App::new("start")
        .arg(clap::Arg::new("project_name").required(true).index(1))
        .arg(clap::Arg::new("task_title").required(true).index(2))
        .arg(clap::Arg::new("tags").index(3).multiple_values(true)),
    )
    .subcommand(clap::App::new("stop"))
    .subcommand(clap::App::new("log").arg(clap::Arg::new("full").long("full")))
    .subcommand(clap::App::new("projects"))
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
          log_task(
            &task,
            pomidorka.project_by_id(task.project_id()).unwrap().name(),
            true,
          );
        }
        Err(err) => println!("couldn't stop: {}", err),
      };
    }

    Some("log") => {
      let full = matches
        .subcommand_matches("log")
        .unwrap()
        .is_present("full");
      log_tasks_list(&pomidorka, Some(chrono::Duration::days(7)), full);
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

  let mut date = None;
  for t in tasks {
    let task_date = t.start_time().date();
    if date.is_none() || date.unwrap() != task_date {
      let msg = format!("\n{}", task_date.format("%A %d %B %Y"),);
      println!("{}", msg.cyan());
      date = Some(task_date);
    }
    let mut project_name = "default".to_owned();
    if let Some(task_project) = pomidorka.project_by_id(t.project_id()) {
      project_name = task_project.name().to_owned();
    }
    log_task(&t, project_name.as_str(), full);
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
    "{}{:04}  {} to {} {:11}  {:8}  {}  [{}]",
    " ".repeat(5),
    task.id(),
    task
      .start_time()
      .naive_local()
      .format("%H:%M")
      .to_string()
      .green(),
    colored_stop_time_msg,
    format_duration(task.duration()),
    project_name.red(),
    match full {
      true => task.title().purple().to_string(),
      false => "".to_owned(),
    },
    tags_str.italic()
  )
}

fn print_projects(pomidorka: &Pomidorka) {
  for project in pomidorka.projects() {
    println!("{}: {}", project.id(), project.name());
  }
}
