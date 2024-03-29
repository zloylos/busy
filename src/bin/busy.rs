extern crate chrono;
extern crate clap;
extern crate clap_complete;
extern crate colored;
extern crate serde;
extern crate serde_json;
extern crate uuid;

mod view;

use std::{
  cell::RefCell,
  collections::HashSet,
  io::{Read, Seek, Write},
  rc::Rc,
};

use busy::{
  duration::{get_midnight_datetime, get_period_since_now, get_week_start_datetime, Period},
  Busy,
};

use busy::task::Task;
use busy::task::TaskView;
use busy::time::parse_datetime;
use busy::traits::Indexable;
use clap::{Arg, ArgMatches, Command};
use colored::Colorize;
use log::debug;
use view::viewer::Viewer;

fn build_cli() -> Command<'static> {
  let command = Command::new("busy")
    .about("Simple CLI time tracker")
    .arg_required_else_help(true)
    .trailing_var_arg(true)
    .subcommand(
      Command::new("add").about("add finished task").args(&[
        Arg::new("project_name").required(true).index(1),
        Arg::new("task_title").required(true).index(2),
        Arg::new("tags")
          .help("should be prefixed with `+` like: +my-tag1 +mytag2")
          .index(3)
          .multiple_values(true),
        Arg::new("start-time")
          .long("start-time")
          .required(true)
          .takes_value(true)
          .help("task start-time, format: HH:MM or YYYY-mm-dd HH:MM"),
        Arg::new("finish-time")
          .long("finish-time")
          .required(true)
          .takes_value(true)
          .help("task finish-time, format: HH:MM or YYYY-mm-dd HH:MM"),
      ]),
    )
    .subcommand(
      Command::new("start").about("start new task").args(&[
        Arg::new("project_name").required(true).index(1),
        Arg::new("task_title").required(true).index(2),
        Arg::new("tags")
          .help("should be prefixed with `+` like: +my-tag1 +mytag2")
          .index(3)
          .multiple_values(true),
        Arg::new("start-time")
          .long("start-time")
          .takes_value(true)
          .help("override start-time, format: HH:MM"),
      ]),
    )
    .subcommand(
      Command::new("status")
        .alias("st")
        .about("show active task if exists"),
    )
    .subcommand(Command::new("stop").about("stop current task"))
    .subcommand(
      Command::new("sync")
        .about("sync tasks. please set $BUSY_REMOTE env")
        .args(&[
          Arg::new("push-force").long("push-force"),
          Arg::new("pull-force").long("pull-force"),
        ]),
    )
    .subcommand(Command::new("pause").about("pause the current task"))
    .subcommand(Command::new("resume").about("resume the current task"))
    .subcommand(
      Command::new("today")
        .alias("td")
        .about("show today tasks, shortcut for `log --today`")
        .args(&[
          Arg::new("full").long("full"),
          Arg::new("dont-clear").long("dont-clear"),
          Arg::new("project")
            .long("project")
            .multiple_values(true)
            .takes_value(true),
          Arg::new("tag")
            .long("tag")
            .multiple_values(true)
            .takes_value(true),
        ]),
    )
    .subcommand(
      Command::new("log").about("print last tasks").args(&[
        Arg::new("days").long("days").takes_value(true),
        Arg::new("full").long("full"),
        Arg::new("today").long("today"),
        Arg::new("dont-clear").long("dont-clear"),
        Arg::new("project")
          .long("project")
          .multiple_values(true)
          .takes_value(true),
        Arg::new("tag")
          .long("tag")
          .multiple_values(true)
          .takes_value(true),
      ]),
    )
    .subcommand(
      Command::new("stat")
        .about("print projects & tags statistic")
        .args(&[
          Arg::new("days").long("days").takes_value(true),
          Arg::new("today").long("today"),
          Arg::new("with-tags").long("with-tags"),
          Arg::new("project")
            .long("project")
            .multiple_values(true)
            .takes_value(true),
          Arg::new("tag")
            .long("tag")
            .multiple_values(true)
            .takes_value(true),
        ]),
    )
    .subcommand(
      Command::new("continue")
        .about("continue specific task (clone and start from now again")
        .args(&[Arg::new("short-task-id").index(1)]),
    )
    .subcommand(
      Command::new("rm")
        .about("remove specific task")
        .args(&[Arg::new("short-task-id").index(1)]),
    )
    .subcommand(Command::new("projects").about("print all projects"))
    .subcommand(Command::new("tags").about("print all tags"))
    .subcommand(
      Command::new("edit").args(&[
        Arg::new("all").long("all").short('a'),
        Arg::new("all-tags").long("all-tags"),
        Arg::new("task-id")
          .long("task")
          .multiple_occurrences(true)
          .takes_value(true),
        Arg::new("project-id")
          .long("project")
          .multiple_occurrences(true)
          .takes_value(true),
        Arg::new("tag-id")
          .long("tag")
          .multiple_occurrences(true)
          .takes_value(true),
      ]),
    );
  return command;
}

fn main() {
  env_logger::init();

  let busy = Rc::new(RefCell::new(Busy::new()));
  let cmd = build_cli();
  let matches = cmd.get_matches();
  let viewer = Viewer::new(Rc::clone(&busy));

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
      match busy.borrow().active_task() {
        Some(task) => {
          println!("Your active task: ");
          viewer.log_task(&task, true);
        }
        None => {
          println!("There are no active tasks");
        }
      };
    }

    Some("add") => {
      let command_matches = matches.subcommand_matches("add").unwrap();
      let project_name = command_matches.value_of("project_name").unwrap();
      let task_title = command_matches.value_of("task_title").unwrap();
      let tags = extract_tags("tags", command_matches);

      let start_time = parse_datetime(command_matches.value_of("start-time").unwrap());
      let finish_time = parse_datetime(command_matches.value_of("finish-time").unwrap());

      if start_time.is_err() || finish_time.is_err() {
        println!(
          "failed to parse start or finish time: {:?} {:?}",
          start_time.err(),
          finish_time.err()
        );
        return;
      }

      let started_task_result = {
        busy.borrow_mut().add(
          project_name,
          task_title,
          tags,
          start_time.unwrap(),
          finish_time.unwrap(),
        )
      };
      match started_task_result {
        Ok(task) => {
          println!("Task added: ");
          viewer.log_task(&task, true);
        }
        Err(err) => println!("add task err: {}", err),
      };
    }

    Some("start") => {
      let command_matches = matches.subcommand_matches("start").unwrap();
      let project_name = command_matches.value_of("project_name").unwrap();
      let task_title = command_matches.value_of("task_title").unwrap();
      let tags = extract_tags("tags", command_matches);
      let start_time_str = command_matches.value_of("start-time");
      let mut start_time = None;
      if start_time_str.is_some() {
        let parsed_start_time = parse_datetime(start_time_str.unwrap());
        if parsed_start_time.is_err() {
          println!(
            "Can't parse start-time parameter: {}, err: {:?}",
            start_time_str.unwrap(),
            parsed_start_time.err()
          );
          return;
        }
        start_time = Some(parsed_start_time.unwrap());
      }

      let started_task_result = {
        busy
          .borrow_mut()
          .start(project_name, task_title, tags, start_time)
      };
      match started_task_result {
        Ok(task) => {
          println!("Task started: ");
          viewer.log_task(&task, true);
        }
        Err(err) => println!("start task err: {}", err),
      };
    }

    Some("sync") => {
      let command_matches = matches.subcommand_matches("sync").unwrap();
      let push_force = command_matches.is_present("push-force");
      let pull_force = command_matches.is_present("pull-force");

      if push_force {
        println!("Start sync push force…");
        match busy.borrow_mut().push_force() {
          Ok(_) => println!("Sync push force success!"),
          Err(err) => println!("Sync push force failed, output:\n{}", err),
        };
      } else if pull_force {
        println!("Start sync pull force…");
        match busy.borrow_mut().pull_force() {
          Ok(_) => println!("Sync pull force success!"),
          Err(err) => println!("Sync pull force failed, output:\n{}", err),
        };
      } else {
        println!("Start syncing…");
        let sync_result = busy.borrow_mut().sync();
        match sync_result {
          Ok(_) => {
            println!("Syncing finished");
          }
          Err(err) => {
            println!("Sync failed, err output:\n{}", err);
            println!("You can try to use `busy sync --push-force` or `busy sync --pull-force`");
          }
        };
      }
    }

    Some("stop") => {
      let stopped_task_result = { busy.borrow_mut().stop() };
      match stopped_task_result {
        Ok(task) => {
          println!("Task stopped:");
          viewer.log_task(&task, true);
        }
        Err(err) => println!("couldn't stop: {}", err),
      };
    }

    Some("pause") => {
      let paused_task_result = { busy.borrow_mut().pause() };
      match paused_task_result {
        Ok(task) => {
          println!("Task paused:");
          viewer.log_task(&task, true);
        }
        Err(err) => println!("couldn't pause: {}", err),
      };
    }

    Some("resume") => {
      let unpaused_task_result = { busy.borrow_mut().resume() };
      match unpaused_task_result {
        Ok(task) => {
          println!("Task resumed:");
          viewer.log_task(&task, true);
        }
        Err(err) => println!("couldn't resume: {}", err),
      };
    }

    Some("log") => {
      let subcommand_matches = matches.subcommand_matches("log").unwrap();
      show_tasks(
        subcommand_matches,
        Rc::clone(&busy),
        &viewer,
        get_period(subcommand_matches),
      );
    }

    Some("today") => {
      show_tasks(
        matches.subcommand_matches("today").unwrap(),
        Rc::clone(&busy),
        &viewer,
        Period::new_to_now(get_midnight_datetime()),
      );
    }

    Some("stat") => {
      clear_screen();
      let subcommand_matches = matches.subcommand_matches("stat").unwrap();
      let with_tags = subcommand_matches.is_present("with-tags");
      let project_names = subcommand_matches
        .values_of_t("project")
        .ok()
        .unwrap_or_default();
      let project_ids = projects_to_ids_set(Rc::clone(&busy), project_names);
      let tags = extract_tags("tag", subcommand_matches);
      let found_tags = busy.borrow().find_tag_by_names(&tags);

      viewer.show_stat(
        get_period(subcommand_matches),
        project_ids,
        &found_tags,
        with_tags,
      );
    }

    Some("continue") => {
      let subcommand_matches = matches.subcommand_matches("continue").unwrap();
      let short_task_id = subcommand_matches.value_of("short-task-id").unwrap();
      let task_id = restore_id_by_short_id(Rc::clone(&busy), short_task_id);
      if task_id.is_err() {
        println!(
          "Continue parse short id into uuid error: {:?}",
          task_id.err()
        );
        return;
      }

      let task = busy.borrow_mut().continue_task(task_id.unwrap());
      if task.is_err() {
        println!("Continue task error: {:?}", task.err());
        return;
      }

      println!("Continue task:");
      viewer.log_task(task.as_ref().unwrap(), true);
    }

    Some("rm") => {
      let subcommand_matches = matches.subcommand_matches("rm").unwrap();
      let short_task_id = subcommand_matches.value_of("short-task-id").unwrap();
      let task_id = restore_id_by_short_id(Rc::clone(&busy), short_task_id);
      if task_id.is_err() {
        println!("Parse short id into uuid error: {:?}", task_id.err());
        return;
      }

      let task: Task;
      {
        let mut p = busy.borrow_mut();
        task = p.task_by_id(task_id.unwrap()).unwrap();
        p.remove_task(task.id()).unwrap();
      };
      println!("Removed task:");
      viewer.log_task(&task, true);
    }

    Some("edit") => {
      let subcommand_matches = matches.subcommand_matches("edit").unwrap();
      if subcommand_matches.is_present("all-tags") {
        edit(
          Rc::clone(&busy),
          &viewer,
          EditDataType::AllTags,
          uuid::Uuid::new_v4(),
        );
        return;
      }

      if subcommand_matches.is_present("all") {
        edit(
          Rc::clone(&busy),
          &viewer,
          EditDataType::AllTasks,
          uuid::Uuid::new_v4(),
        );
        return;
      }

      let extract_ids_and_edit = |name: &str, edit_type: EditDataType| {
        let short_item_ids: Vec<String> = subcommand_matches.values_of_t(name).unwrap_or_default();
        let ids: Vec<uuid::Uuid> = short_item_ids
          .iter()
          .map(|short_id| restore_id_by_short_id(Rc::clone(&busy), short_id).unwrap())
          .collect();

        for id in ids {
          edit(Rc::clone(&busy), &viewer, edit_type, id);
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

fn show_tasks(
  subcommand_matches: &ArgMatches,
  busy: Rc<RefCell<Busy>>,
  viewer: &Viewer,
  period: Period,
) {
  if !subcommand_matches.is_present("dont-clear") {
    clear_screen();
  }

  let show_full = subcommand_matches.is_present("full");
  let project_names = subcommand_matches
    .values_of_t("project")
    .ok()
    .unwrap_or_default();
  let project_ids = projects_to_ids_set(Rc::clone(&busy), project_names);
  let tags = extract_tags("tag", subcommand_matches);
  let found_tags = busy.borrow().find_tag_by_names(&tags);

  viewer.log_tasks_list(period, project_ids, &found_tags, show_full);
}

#[derive(Debug, Clone, Copy)]
enum EditDataType {
  Task,
  Project,
  Tag,
  AllTags,
  AllTasks,
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

fn edit(busy: Rc<RefCell<Busy>>, viewer: &Viewer, edit_data_type: EditDataType, id: uuid::Uuid) {
  let editor = get_editor();
  let mut tmp_file = tempfile::Builder::new()
    .prefix("busy_")
    .suffix(".json")
    .tempfile()
    .unwrap();

  debug!(
    "edit {:?} id: {} tmp_file_path: {:?}",
    edit_data_type, id, tmp_file
  );

  match edit_data_type {
    EditDataType::Task => {
      let task = busy.borrow().task_by_id(id).unwrap();
      let mut all_tags = busy.borrow().tags();
      let task_view = TaskView::from_task(&task, &all_tags);

      let updated_task_view = run_edit_and_get_result(&task_view, &mut tmp_file, &editor);

      let new_tags = updated_task_view.resolve_new_tags(&all_tags);
      busy.borrow_mut().upsert_tags(new_tags);
      all_tags = busy.borrow().tags();

      let updated_task = updated_task_view.to_task(&all_tags);
      viewer.log_task(&updated_task, true);
      busy.borrow_mut().replace_task(&updated_task).unwrap();
    }

    EditDataType::Project => {
      let project = busy.borrow().project_by_id(id).unwrap();
      let updated_project = run_edit_and_get_result(&project, &mut tmp_file, &editor);

      println!("{}", "Updated project: ".bright_yellow());
      viewer.print_project(&updated_project);
      busy.borrow_mut().replace_project(&updated_project).unwrap();
    }

    EditDataType::Tag => {
      let tag = busy.borrow().tag_by_id(id).unwrap();
      let updated_tag = run_edit_and_get_result(&tag, &mut tmp_file, &editor);

      println!("{}", "Updated tag: ".bright_yellow());
      viewer.print_tag(&updated_tag);
      busy.borrow_mut().replace_tag(&updated_tag).unwrap();
    }

    EditDataType::AllTags => {
      let edited_data =
        run_edit_and_get_result(&busy.borrow().all_tags(), &mut tmp_file, editor.as_str());
      busy.borrow_mut().replace_tags(edited_data);
      println!("Edit finished, tags were saved");
    }

    EditDataType::AllTasks => {
      let edited_data =
        run_edit_and_get_result(&busy.borrow().all_tasks(), &mut tmp_file, editor.as_str());
      busy.borrow_mut().replace_tasks(edited_data);
      println!("Edit finished, tasks were saved");
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
  busy: Rc<RefCell<Busy>>,
  project_names: Vec<String>,
) -> Option<HashSet<uuid::Uuid>> {
  let mut project_ids = HashSet::new();
  for project_name in project_names.iter() {
    let project = busy.borrow().project_by_name(project_name);
    if project.is_some() {
      project_ids.insert(project.unwrap().id().clone());
    }
  }
  if project_ids.is_empty() {
    return None;
  }
  return Some(project_ids);
}

fn get_period(subcommand_matches: &ArgMatches) -> Period {
  let show_today_only = subcommand_matches.is_present("today");
  if show_today_only {
    return Period::new_to_now(get_midnight_datetime());
  }

  let period_days = subcommand_matches.value_of_t("days").ok();
  if period_days.is_none() {
    return Period::new_to_now(get_week_start_datetime());
  }

  return Period::new_to_now(get_period_since_now(period_days.unwrap()));
}

fn restore_id_by_short_id(busy: Rc<RefCell<Busy>>, short_id: &str) -> Result<uuid::Uuid, String> {
  match busy.borrow().resolve_id(short_id) {
    Some(id) => Ok(id.clone()),
    None => Err(format!("id by short name: {} not found", short_id)),
  }
}
