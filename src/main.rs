extern crate chrono;
extern crate colored;
extern crate serde;
extern crate serde_json;

use std::{
  cell::RefCell,
  collections::HashSet,
  io::{Read, Seek, Write},
  rc::Rc,
};

use chrono::{Datelike, Timelike};
use clap::ArgMatches;
use colored::Colorize;
use log::debug;
use task::TaskView;
use traits::Indexable;
use viewer::Viewer;

use crate::{pomidorka::Pomidorka, task::Task};

mod duration_fmt;
mod pomidorka;
mod project;
mod state;
mod storage;
mod tag;
mod task;
mod traits;
mod viewer;

fn build_cli(_: Rc<RefCell<Pomidorka>>) -> clap::Command<'static> {
  clap::Command::new("pomidorka")
    .about("Simple CLI time tracker")
    .arg_required_else_help(true)
    .trailing_var_arg(true)
    .subcommand(
      clap::Command::new("start").about("start new task").args(&[
        clap::Arg::new("project_name").required(true).index(1),
        clap::Arg::new("task_title").required(true).index(2),
        clap::Arg::new("tags")
          .help("should be prefixed with `+` like: +my-tag1 +mytag2")
          .index(3)
          .multiple_values(true),
      ]),
    )
    .subcommand(clap::Command::new("status").about("show active task if exists"))
    .subcommand(clap::Command::new("stop").about("stop current task"))
    .subcommand(
      clap::Command::new("log").about("print last tasks").args(&[
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
      clap::Command::new("stat")
        .about("print projects & tags statistics")
        .args(&[
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
    .subcommand(
      clap::Command::new("rm")
        .about("remove specific task")
        .args(&[clap::Arg::new("task-id").index(1)]),
    )
    .subcommand(clap::Command::new("projects").about("print all projects"))
    .subcommand(clap::Command::new("tags").about("print all tags"))
    .subcommand(
      clap::Command::new("edit").args(&[
        clap::Arg::new("all").long("all").short('a'),
        clap::Arg::new("task-id")
          .long("task")
          .multiple_occurrences(true)
          .takes_value(true),
        clap::Arg::new("project-id")
          .long("project")
          .multiple_occurrences(true)
          .takes_value(true),
        clap::Arg::new("tag-id")
          .long("tag")
          .multiple_occurrences(true)
          .takes_value(true),
      ]),
    )
}

fn main() {
  env_logger::init();

  let pomidorka = Rc::new(RefCell::new(Pomidorka::new()));

  let cmd = build_cli(Rc::clone(&pomidorka));
  let matches = cmd.get_matches();

  let viewer = Viewer::new(Rc::clone(&pomidorka));
  match matches.subcommand_name() {
    Some("projects") => {
      clear_screen();
      println!("{}", "Projects: ".bright_cyan());
      viewer.print_projects();
    }

    Some("tags") => {
      clear_screen();
      println!("{}", "Tags: ".bright_cyan());
      viewer.print_tags();
    }

    Some("status") => {
      match pomidorka.borrow().active_task() {
        Some(task) => {
          println!("Your active task: ");
          viewer.log_task(&task, true);
        }
        None => {
          println!("There are no active tasks");
        }
      };
    }

    Some("start") => {
      let command_matches = matches.subcommand_matches("start").unwrap();
      let project_name = command_matches.value_of("project_name").unwrap();
      let task_title = command_matches.value_of("task_title").unwrap();
      let tags = extract_tags("tags", command_matches);

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
      let tags = extract_tags("tag", subcommand_matches);

      let found_tags = pomidorka.borrow().find_tag_by_names(&tags);
      let period_arg = subcommand_matches.value_of_t("days").ok();
      let period = get_period(period_arg, show_today_only);
      viewer.log_tasks_list(period, project_ids, &found_tags, show_full);
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
      let tags = extract_tags("tag", subcommand_matches);

      let found_tags = pomidorka.borrow().find_tag_by_names(&tags);
      let period_arg = subcommand_matches.value_of_t("days").ok();
      let period = get_period(period_arg, show_today_only);
      viewer.show_stat(period, project_ids, &found_tags, with_tags);
    }

    Some("rm") => {
      let subcommand_matches = matches.subcommand_matches("rm").unwrap();
      let task_id: u128 = subcommand_matches.value_of_t("task-id").unwrap();
      let task: Task;
      {
        let mut p = pomidorka.borrow_mut();
        task = p.task_by_id(task_id).unwrap();
        p.remove_task(task_id).unwrap();
      };
      println!("Removed task:");
      viewer.log_task(&task, true);
    }

    Some("edit") => {
      let subcommand_matches = matches.subcommand_matches("edit").unwrap();
      if subcommand_matches.is_present("all") {
        edit(Rc::clone(&pomidorka), &viewer, EditDataType::All, 0);
        return;
      }

      let extract_ids_and_edit = |name: &str, edit_type: EditDataType| {
        let item_ids: Vec<u128> = subcommand_matches.values_of_t(name).unwrap_or_default();
        for id in item_ids {
          edit(Rc::clone(&pomidorka), &viewer, edit_type, id);
        }
      };

      extract_ids_and_edit("task-id", EditDataType::Task);
      extract_ids_and_edit("project-id", EditDataType::Project);
      extract_ids_and_edit("tag-id", EditDataType::Tag);

      println!("\nEdit completed");
    }

    Some(subcmd) => println!("unknown subcommand {}", subcmd),

    None => println!("subcommand not found"),
  };
}

#[derive(Debug, Clone, Copy)]
enum EditDataType {
  Task,
  Project,
  Tag,
  All,
}

fn get_editor() -> String {
  std::env::var("EDITOR").unwrap_or(std::env::var("VISUAL").unwrap_or("nvim".to_string()))
}

fn run_edit_and_get_result<T: serde::ser::Serialize + serde::de::DeserializeOwned>(
  item: &T,
  tmp_file: &mut tempfile::NamedTempFile,
  editor: &str,
) -> T {
  let item_str = serde_json::to_string_pretty(item).unwrap();
  tmp_file.write_all(item_str.as_bytes()).unwrap();

  subprocess::Exec::cmd(editor)
    .arg(tmp_file.path())
    .join()
    .expect("edit cmd doesn't work");

  let mut buf = String::new();
  tmp_file.seek(std::io::SeekFrom::Start(0)).unwrap();
  tmp_file.read_to_string(&mut buf).unwrap();

  debug!("edit result: {}", buf);

  return serde_json::from_str(&buf).expect("can't decode item back, please try again");
}

fn edit(
  pomidorka: Rc<RefCell<Pomidorka>>,
  viewer: &Viewer,
  edit_data_type: EditDataType,
  id: u128,
) {
  let editor = get_editor();
  let mut tmp_file = tempfile::Builder::new()
    .prefix("pomidorka_")
    .suffix(".json")
    .tempfile()
    .unwrap();

  debug!(
    "edit {:?} id: {} tmp_file_path: {:?}",
    edit_data_type, id, tmp_file
  );

  match edit_data_type {
    EditDataType::Task => {
      let task = pomidorka.borrow().task_by_id(id).unwrap();
      let all_tags = pomidorka.borrow().tags();
      let task_view = TaskView::from_task(&task, &all_tags);

      let updated_task_view = run_edit_and_get_result(&task_view, &mut tmp_file, &editor);
      let updated_task = updated_task_view.to_task(&all_tags);
      viewer.log_task(&updated_task, true);
      pomidorka.borrow_mut().replace_task(updated_task).unwrap();
    }

    EditDataType::Project => {
      let project = pomidorka.borrow().project_by_id(id).unwrap();
      let updated_project = run_edit_and_get_result(&project, &mut tmp_file, &editor);

      println!("{}", "Updated project: ".bright_yellow());
      viewer.print_project(&updated_project);
      pomidorka
        .borrow_mut()
        .replace_project(updated_project)
        .unwrap();
    }

    EditDataType::Tag => {
      let tag = pomidorka.borrow().tag_by_id(id).unwrap();
      let updated_tag = run_edit_and_get_result(&tag, &mut tmp_file, &editor);

      println!("{}", "Updated tag: ".bright_yellow());
      viewer.print_tag(&updated_tag);
      pomidorka.borrow_mut().replace_tag(updated_tag).unwrap();
    }

    EditDataType::All => {
      let filepath = pomidorka.borrow().tasks_db_filepath().to_string();
      subprocess::Exec::cmd(&editor).arg(filepath).join().unwrap();
    }
  };
}

fn clear_screen() {
  if log::log_enabled!(log::Level::Debug) {
    return;
  }
  subprocess::Exec::cmd("clear")
    .join()
    .expect("clean cmd doesn't work");
}

fn extract_tags(values_of_t: &str, command_matches: &ArgMatches) -> Vec<String> {
  let tags: Vec<String> = command_matches
    .values_of_t(values_of_t)
    .unwrap_or_default()
    .iter_mut()
    .map(|tag: &mut String| tag.strip_prefix("+").unwrap_or(tag).to_string())
    .collect();

  return tags;
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
  let seconds_from_midnight = chrono::Duration::seconds(
    chrono::Local::now()
      .time()
      .num_seconds_from_midnight()
      .into(),
  );

  if period_days.is_some() {
    return chrono::Duration::days(period_days.unwrap())
      .checked_add(&seconds_from_midnight)
      .unwrap();
  }

  return match show_today_only {
    true => seconds_from_midnight,
    false => chrono::Duration::days(chrono::Local::now().weekday().num_days_from_monday() as i64)
      .checked_add(&seconds_from_midnight)
      .unwrap(),
  };
}
