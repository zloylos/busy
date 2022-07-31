use std::{cell::RefCell, rc::Rc};

use colored::Colorize;

use crate::{
  duration_fmt::{format_duration, format_duration_without_paddings},
  pomidorka::Pomidorka,
  task,
};

pub struct Viewer {
  pomidorka: Rc<RefCell<Pomidorka>>,
}

impl Viewer {
  pub fn new(pomidorka: Rc<RefCell<Pomidorka>>) -> Self {
    Self { pomidorka }
  }

  pub fn print_projects(&self) {
    for project in self.pomidorka.borrow().projects() {
      println!("{}: {}", project.id(), project.name());
    }
  }

  pub fn log_tasks_list(&self, period: Option<chrono::Duration>, show_full: bool) {
    let tasks = self.pomidorka.borrow().tasks(period.unwrap());
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
        self.log_task(t, show_full);
      }
      println!("");
    }
  }

  fn get_project_name(&self, project_id: u128) -> String {
    if let Some(task_project) = self.pomidorka.borrow().project_by_id(project_id) {
      return task_project.name().to_string();
    }
    return "default".to_string();
  }

  pub fn log_task(&self, task: &task::Task, show_full: bool) {
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
        project = self.get_project_name(task.project_id()).as_str().red(),
        tags = tags_str.italic()
      )
    );
    if show_full {
      println!(
        "{}{}",
        " ".repeat(4 + 4 + 32),
        task.title().dimmed().italic()
      );
    }
  }
}
