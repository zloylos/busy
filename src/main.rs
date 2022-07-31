extern crate chrono;
extern crate serde;
extern crate serde_json;

use crate::pomidorka::Pomidorka;
use chrono::Datelike;
use clap::App;

mod category;
mod pomidorka;
mod state;
mod storage;
mod task;

fn main() {
  let matches = App::new("Pomidorka")
    .subcommand(
      clap::App::new("categories")
        .subcommand(
          clap::App::new("add").arg(
            clap::Arg::new("name")
              .about("category name")
              .required(true)
              .index(1),
          ),
        )
        .subcommand(clap::App::new("list")),
    )
    .subcommand(
      clap::App::new("tasks")
        .subcommand(
          clap::App::new("list")
            .arg(clap::Arg::new("active_only").long("active-only").short('a'))
            .arg(
              clap::Arg::new("period")
                .about("Period in days")
                .default_value("7"),
            ),
        )
        .subcommand(
          clap::App::new("add")
            .arg(
              clap::Arg::new("title")
                .about("task title")
                .required(true)
                .index(1),
            )
            .arg(
              clap::Arg::new("category")
                .long("category")
                .short('c')
                .default_value("0"),
            ),
        )
        .subcommand(clap::App::new("remove").arg(clap::Arg::new("task_id").required(true).index(1)))
        .subcommand(clap::App::new("stop").arg(clap::Arg::new("task_id").required(true).index(1))),
    )
    .get_matches();

  let mut pomidorka = Pomidorka::new();
  match matches.subcommand_name() {
    Some("categories") => {
      let categories_subcommand_matches = matches.subcommand_matches("categories").unwrap();
      match categories_subcommand_matches.subcommand_name() {
        Some("add") => {
          let category_name = categories_subcommand_matches
            .subcommand_matches("add")
            .unwrap()
            .value_of("name")
            .unwrap();

          pomidorka.add_category(category_name);
          println!("category added, list of categories: ");
          print_categories(&pomidorka);
        }

        Some("list") => {
          print_categories(&pomidorka);
        }

        Some(subcmd) => println!("unknown task subcommand {}", subcmd),

        None => print_categories(&pomidorka),
      }
    }

    Some("tasks") => {
      let tasks_subcommand_matches = matches.subcommand_matches("tasks").unwrap();
      match tasks_subcommand_matches.subcommand_name() {
        Some("add") => {
          let add_matches = tasks_subcommand_matches.subcommand_matches("add").unwrap();
          let title = add_matches.value_of("title").unwrap();
          let category_id: u128 = add_matches.value_of_t("category").unwrap();

          println!(
            "add task with category id: {} / {}",
            category_id,
            pomidorka.category_by_id(category_id).unwrap().name()
          );

          pomidorka.add_task(category_id, title);
          println!("task added, list of tasks: ");
          print_tasks_list(&pomidorka, true, None);
        }

        Some("remove") => {
          let task_id_str = tasks_subcommand_matches
            .subcommand_matches("remove")
            .unwrap()
            .value_of("task_id")
            .unwrap();

          match pomidorka.remove_task(task_id_str.parse().unwrap()) {
            Ok(_) => println!("task removed"),
            Err(e) => println!("task remove err: {}", e),
          };
        }

        Some("stop") => {
          let task_id: u128 = tasks_subcommand_matches
            .subcommand_matches("stop")
            .unwrap()
            .value_of_t("task_id")
            .unwrap();

          match pomidorka.stop_task(task_id) {
            Ok(_) => println!("task stopped"),
            Err(e) => println!("task stop err: {}", e),
          };
        }

        Some("list") => {
          let list_matches = tasks_subcommand_matches.subcommand_matches("list").unwrap();
          let only_active_flag = list_matches.is_present("active_only");
          let period: i64 = list_matches.value_of_t("period").unwrap();
          print_tasks_list(
            &pomidorka,
            only_active_flag,
            Some(chrono::Duration::days(period)),
          );
        }

        Some(subcmd) => println!("unknown task subcommand {}", subcmd),

        None => print_tasks_list(&pomidorka, false, Some(chrono::Duration::days(7))),
      };
    }
    Some(subcmd) => println!("unknown subcommand {}", subcmd),
    None => println!("subcommand not found"),
  };
}

fn print_tasks_list(pomidorka: &Pomidorka, only_active: bool, period: Option<chrono::Duration>) {
  let tasks = match only_active {
    true => pomidorka.active_tasks(),
    false => pomidorka.tasks(period.unwrap()),
  };
  if tasks.is_empty() {
    println!("no tasks to show");
    return;
  }

  let mut date = None;
  for t in tasks {
    let task_date = t.start_time().date();
    if date.is_none() || date.unwrap() != task_date {
      println!(
        "\n{}: {}",
        task_date.format("%Y-%m-%d"),
        task_date.weekday()
      );
      println!("{}", "—".repeat(50));
      date = Some(task_date);
    }
    let mut category_name = "default".to_owned();
    if let Some(task_category) = pomidorka.category_by_id(t.category_id()) {
      category_name = task_category.name().to_owned();
    }
    print_task(&t, category_name.as_str());
  }
}

fn print_task(task: &task::Task, category_name: &str) {
  let time_left = chrono::Duration::from_std(task.time_left()).unwrap();
  let mut time_left_str = "".to_owned();
  if !time_left.is_zero() {
    time_left_str = format!(
      ": {:02}:{:02}",
      time_left.num_minutes(),
      time_left.num_seconds() % 60
    );
  }

  let task_duration = chrono::Duration::from_std(task.duration()).unwrap();
  println!(
    "#{:04} | {} — {} | {:8} |  {} {}",
    task.id(),
    task.start_time().naive_local().format("%H:%M"),
    (task.start_time() + task_duration)
      .naive_local()
      .format("%H:%M"),
    category_name,
    task.description(),
    time_left_str,
  )
}

fn print_categories(pomidorka: &Pomidorka) {
  for category in pomidorka.categories() {
    println!("{}: {}", category.id(), category.name());
  }
}
