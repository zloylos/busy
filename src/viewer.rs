use std::{
  cell::RefCell,
  collections::{BTreeMap, BTreeSet, HashMap, HashSet},
  rc::Rc,
};

use colored::{ColoredString, Colorize};

use crate::{
  duration_fmt::{format_duration, format_duration_without_paddings},
  pomidorka::Pomidorka,
  project::Project,
  tag::Tag,
  task::{self, DateTimeInterval, Task},
  traits::Indexable,
};

pub struct Viewer {
  pomidorka: Rc<RefCell<Pomidorka>>,
}

impl Viewer {
  pub fn new(pomidorka: Rc<RefCell<Pomidorka>>) -> Self {
    Self { pomidorka }
  }

  pub fn print_tag(&self, tag: &Tag) {
    println!("id: {}, {}", tag.id(), tag.name());
  }

  pub fn print_tags(&self) {
    for tag in self.pomidorka.borrow().tags() {
      self.print_tag(&tag);
    }
  }

  pub fn print_project(&self, project: &Project) {
    println!("id: {}, {}", project.id(), project.name());
  }

  pub fn print_projects(&self) {
    for project in self.pomidorka.borrow().projects() {
      self.print_project(&project);
    }
  }

  pub fn show_stat(
    &self,
    period: chrono::Duration,
    project_ids: Option<HashSet<u128>>,
    tags: &Vec<Tag>,
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
        let task_tags = self.pomidorka.borrow().storage().find_tags(task.tags());

        for tag in task_tags {
          let tag_duration = tag_times
            .entry(tag.name().to_string())
            .or_insert(chrono::Duration::zero());
          *tag_duration = tag_duration.clone().checked_add(&task.duration()).unwrap();
          project_tags.insert(tag.name().to_string());
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
    tags: &Vec<Tag>,
  ) -> Vec<Vec<Task>> {
    let tasks = self.pomidorka.borrow().tasks(period);
    if tasks.is_empty() {
      return Vec::new();
    }

    let mut by_dates: Vec<Vec<Task>> = Vec::new();
    let mut date = None;
    let has_project_ids = maybe_project_ids.is_some();
    let project_ids = maybe_project_ids.unwrap_or_default();

    for task in tasks {
      if has_project_ids && !project_ids.contains(&task.project_id()) {
        continue;
      }

      if !tags.is_empty() {
        if !task
          .tags()
          .iter()
          .any(|t| tags.iter().position(|tag| tag.id() == *t).is_some())
        {
          continue;
        }
      }

      let task_date = task.start_time().date();
      if date.is_none() || date.unwrap() != task_date {
        by_dates.push(Vec::new());
        date = Some(task_date);
      }
      by_dates.last_mut().unwrap().push(task);
    }
    return by_dates;
  }

  pub fn log_tasks_list(
    &self,
    period: chrono::Duration,
    project_ids: Option<HashSet<u128>>,
    tags: &Vec<Tag>,
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
    let task_tags = self.pomidorka.borrow().storage().find_tags(task.tags());
    let tags: Vec<String> = task_tags
      .iter()
      .map(|tag| tag.name().cyan().to_string())
      .collect();

    let tags_str = tags.join(", ");

    let first_interval = task.times().first().unwrap();
    let first_stop_time = match task.times().len() > 1 {
      true => format_time(&first_interval.stop_time.unwrap()),
      false => format_stop_time(task, first_interval.stop_time),
    };

    let project_name = self.get_project_name(task.project_id());
    let mut project_name_msg = project_name.as_str().red();
    if task.is_paused() {
      project_name_msg = (project_name + " [paused]").yellow();
    }

    println!(
      "{}",
      format!(
        "{padding}{task_id:04}  {start_time} to {stop_time} {duration:11}  {project:10}  [{tags}]",
        padding = " ".repeat(5),
        task_id = task.id(),
        start_time = format_time(&first_interval.start_time).green(),
        stop_time = first_stop_time,
        duration = format_duration(task.duration()),
        project = project_name_msg,
        tags = tags_str.italic()
      )
    );

    let task_description = task.title().dimmed().italic();
    if task.times().len() > 1 {
      let mut task_description_printed = !show_full;

      let mut time_iter = task.times().iter().skip(1);
      let last_interval = time_iter.next_back();

      for time_interval in time_iter {
        print_time_interval(
          time_interval,
          Some(format_time(&time_interval.stop_time.unwrap())),
          match task_description_printed {
            true => None,
            false => Some(task_description.clone()),
          },
        );
        task_description_printed = true;
      }

      if let Some(last_interval) = last_interval {
        print_time_interval(
          last_interval,
          Some(format_stop_time(task, last_interval.stop_time)),
          match task_description_printed {
            true => None,
            false => Some(task_description.clone()),
          },
        );
        println!();
      }
    } else if show_full {
      println!("{}{}", " ".repeat(4 + 4 + 32), task_description);
    }
  }
}

fn format_stop_time(
  task: &Task,
  stop_time: Option<chrono::DateTime<chrono::Local>>,
) -> ColoredString {
  let stop_time_msg = format_time(&stop_time.unwrap_or(chrono::Local::now()));
  if stop_time.is_some() {
    if task.is_paused() {
      return stop_time_msg.bold().italic().bright_red();
    }
    return stop_time_msg.green();
  }
  return stop_time_msg.yellow();
}

fn format_time(time: &chrono::DateTime<chrono::Local>) -> ColoredString {
  return time.naive_local().format("%H:%M").to_string().black();
}

fn print_time_interval(
  time_interval: &DateTimeInterval,
  stop_time_formatted: Option<ColoredString>,
  task_description: Option<ColoredString>,
) {
  println!(
    "{}",
    format!(
      "{padding}  {start_time} to {stop_time} {task_description_padding} {task_description}",
      padding = " ".repeat(5 + 4),
      start_time = format_time(&time_interval.start_time),
      stop_time = stop_time_formatted.unwrap_or(format_time(
        &time_interval.stop_time.unwrap_or(chrono::Local::now())
      )),
      task_description_padding = " ".repeat(13),
      task_description = task_description.unwrap_or_default()
    )
  );
}
