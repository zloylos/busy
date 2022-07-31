use std::{
  cell::RefCell,
  collections::{BTreeMap, BTreeSet, HashMap, HashSet},
  rc::Rc,
};

use colored::Colorize;

use crate::{
  duration_fmt::{format_duration, format_duration_without_paddings},
  pomidorka::Pomidorka,
  task::{self, Task},
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

  pub fn show_stat(
    &self,
    period: chrono::Duration,
    project_ids: Option<HashSet<u128>>,
    tags: &Vec<String>,
    with_tags: bool,
  ) {
    let by_dates = self.tasks_by_day(period, project_ids, tags);
    if by_dates.is_empty() {
      println!("no tasks to show");
      return;
    }

    for tasks in by_dates.iter() {
      self.print_date(tasks);
      let mut project_times: BTreeMap<u128, chrono::Duration> = BTreeMap::new();
      let mut tag_times: HashMap<String, chrono::Duration> = HashMap::new();
      let mut project_to_tags: HashMap<u128, BTreeSet<String>> = HashMap::new();

      for task in tasks {
        let project_id = task.project_id();
        let task_duration = project_times
          .entry(project_id)
          .or_insert(chrono::Duration::zero());
        *task_duration = task_duration.clone().checked_add(&task.duration()).unwrap();

        let project_tags = project_to_tags.entry(project_id).or_insert(BTreeSet::new());
        for tag in task.tags() {
          let tag_duration = tag_times
            .entry(tag.to_string())
            .or_insert(chrono::Duration::zero());
          *tag_duration = tag_duration.clone().checked_add(&task.duration()).unwrap();
          project_tags.insert(tag.to_string());
        }
      }

      for (&project_id, &project_time) in project_times.iter() {
        let mut tags_str = "".to_string();
        if with_tags {
          for tag in project_to_tags.entry(project_id).or_default().iter() {
            tags_str += &format!(
              "\n    + {}: {}",
              tag.bright_yellow().bold(),
              format_duration_without_paddings(*tag_times.get(tag).unwrap())
            );
          }
          tags_str += "\n";
        }

        println!(
          "  {}: {}{}",
          self.get_project_name(project_id).green(),
          format_duration_without_paddings(project_time).bold(),
          tags_str
        );
      }
      if !with_tags {
        println!("");
      }
    }
  }

  fn tasks_by_day(
    &self,
    period: chrono::Duration,
    maybe_project_ids: Option<HashSet<u128>>,
    tags: &Vec<String>,
  ) -> Vec<Vec<Task>> {
    let tasks = self.pomidorka.borrow().tasks(period);
    if tasks.is_empty() {
      return Vec::new();
    }

    let mut by_dates: Vec<Vec<Task>> = Vec::new();
    let mut date = None;
    let has_project_ids = maybe_project_ids.is_some();
    let project_ids = maybe_project_ids.unwrap_or_default();

    for t in tasks {
      if has_project_ids && !project_ids.contains(&t.project_id()) {
        continue;
      }

      if !tags.is_empty() {
        if !t.tags().iter().any(|t| tags.contains(t)) {
          continue;
        }
      }

      let task_date = t.start_time().date();
      if date.is_none() || date.unwrap() != task_date {
        by_dates.push(Vec::new());
        date = Some(task_date);
      }
      by_dates.last_mut().unwrap().push(t);
    }
    return by_dates;
  }

  pub fn log_tasks_list(
    &self,
    period: chrono::Duration,
    project_ids: Option<HashSet<u128>>,
    tags: &Vec<String>,
    show_full: bool,
  ) {
    let by_dates = self.tasks_by_day(period, project_ids, tags);
    if by_dates.is_empty() {
      println!("no tasks to show");
      return;
    }

    for tasks in by_dates.iter() {
      self.print_date(tasks);
      for t in tasks.iter() {
        self.log_task(t, show_full);
      }
      println!("");
    }
  }

  fn total_time(&self, tasks: &Vec<Task>) -> chrono::Duration {
    return tasks
      .iter()
      .map(|t| t.duration())
      .reduce(|acc, new_d| acc + new_d)
      .unwrap_or(chrono::Duration::zero());
  }

  fn print_date(&self, tasks: &Vec<Task>) {
    let date = tasks.first().unwrap().start_time().date();
    let total_time = self.total_time(tasks);
    println!(
      "{} — {}",
      date.format("%A, %d %B %Y").to_string().bold().cyan(),
      format_duration_without_paddings(total_time)
        .bold()
        .bright_yellow()
    );
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
