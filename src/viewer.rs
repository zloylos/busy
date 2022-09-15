use std::{
  cell::RefCell,
  collections::{BTreeMap, BTreeSet, HashMap, HashSet},
  fmt::Display,
  rc::Rc,
};

use colored::{Color, ColoredString, Colorize};

use crate::{
  busy::Busy,
  duration::Period,
  duration_fmt::{format_duration, format_duration_without_paddings},
  project::Project,
  tag::Tag,
  task::{self, DateTimeInterval, Task},
  traits::Indexable,
};

#[derive(Clone, Copy)]
struct Padding(usize);
impl Padding {
  pub fn size(self) -> usize {
    self.0
  }
  pub fn string(self) -> String {
    " ".repeat(self.size())
  }
}

impl Display for Padding {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.string())
  }
}

macro_rules! sum_pads {
  ($($p:expr), *) => {
    {
      let mut sum = 0;
      $(
        sum += $p.0;
      )*
      Padding(sum)
    }
  };
}

struct ViewPaddings {}
impl ViewPaddings {
  pub const PAD: Padding = Padding(2);
  pub const LINE_INDENT: Padding = Padding(4);
  // `dfcf..d73b`
  pub const ID: Padding = Padding(10);
  // `12h 12m to 12h 12m`
  pub const TIME_FRAME: Padding = Padding(5 + 1 + 2 + 1 + 5);
  // `12h 12m`
  pub const DURATION: Padding = Padding(3 + 1 + 3);
  // `work` etc
  pub const TILL_TIME_FRAME: Padding = sum_pads!(Self::LINE_INDENT, Self::ID, Self::PAD);
  pub const TILL_PROJECT: Padding = sum_pads!(
    Self::TILL_TIME_FRAME,
    Self::TIME_FRAME,
    Self::PAD,
    Self::DURATION,
    Self::PAD
  );
  pub const FROM_DURATION_TILL_DESCRIPTION: Padding =
    sum_pads!(Self::PAD, Self::DURATION, Self::PAD);
}

pub struct Viewer {
  busy: Rc<RefCell<Busy>>,
}

impl Viewer {
  pub fn new(busy: Rc<RefCell<Busy>>) -> Self {
    Self { busy }
  }

  pub fn print_tag(&self, tag: &Tag) {
    println!(
      "id: {id}, {tag_name}",
      id = format_id(tag.id()),
      tag_name = tag.name()
    );
  }

  pub fn print_tags(&self) {
    for tag in self.busy.borrow().tags() {
      self.print_tag(&tag);
    }
  }

  pub fn print_project(&self, project: &Project) {
    println!(
      "id: {id}, {project_name}",
      id = format_id(project.id()),
      project_name = project.name()
    );
  }

  pub fn print_projects(&self) {
    for project in self.busy.borrow().projects() {
      self.print_project(&project);
    }
  }

  pub fn show_stat(
    &self,
    period: Period,
    project_ids: Option<HashSet<uuid::Uuid>>,
    tags: &Vec<Tag>,
    with_tags: bool,
  ) {
    let by_dates = self.tasks_by_day(period, project_ids, tags);
    if by_dates.is_empty() {
      println!("no tasks to show");
      return;
    }

    let mut total_duration = chrono::Duration::zero();
    for tasks in by_dates.iter() {
      total_duration = total_duration + self.total_time(tasks);
      self.print_date(tasks);
      let mut project_times: BTreeMap<uuid::Uuid, chrono::Duration> = BTreeMap::new();
      let mut tag_times: HashMap<String, chrono::Duration> = HashMap::new();
      let mut project_to_tags: HashMap<uuid::Uuid, BTreeSet<String>> = HashMap::new();

      for task in tasks {
        let project_id = task.project_id();
        let task_duration = project_times
          .entry(project_id)
          .or_insert(chrono::Duration::zero());
        *task_duration = task_duration.clone().checked_add(&task.duration()).unwrap();

        let project_tags = project_to_tags.entry(project_id).or_insert(BTreeSet::new());
        let task_tags = self.busy.borrow().find_tags(task.tags());

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
              "\n{indent}+ {tag_name}: {duration}",
              indent = ViewPaddings::LINE_INDENT,
              tag_name = tag.bright_yellow().bold(),
              duration = format_duration_without_paddings(*tag_times.get(tag).unwrap())
            );
          }
          tags_str += "\n";
        }

        println!(
          "{indent}{project_name}: {duration}{tags}",
          indent = ViewPaddings::LINE_INDENT,
          project_name = self.get_project_name(project_id).green(),
          duration = format_duration_without_paddings(project_time).bold(),
          tags = tags_str
        );
      }
      if !with_tags {
        println!("");
      }
    }

    println!(
      "Total: {duration}",
      duration = format_duration_without_paddings(total_duration).bold()
    );
  }

  fn tasks_by_day(
    &self,
    period: Period,
    maybe_project_ids: Option<HashSet<uuid::Uuid>>,
    tags: &Vec<Tag>,
  ) -> Vec<Vec<Task>> {
    let tasks = self.busy.borrow().tasks(period);
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
    period: Period,
    project_ids: Option<HashSet<uuid::Uuid>>,
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
      "{date} — {duration}",
      date = date.format("%A, %d %B %Y").to_string().bold().cyan(),
      duration = format_duration_without_paddings(total_time)
        .bold()
        .bright_yellow()
    );
  }

  fn get_project_name(&self, project_id: uuid::Uuid) -> String {
    if let Some(task_project) = self.busy.borrow().project_by_id(project_id) {
      return task_project.name().to_string();
    }
    return "default".to_string();
  }

  pub fn log_task(&self, task: &task::Task, show_full: bool) {
    let task_tags = self.busy.borrow().find_tags(task.tags());
    let tags: Vec<String> = task_tags
      .iter()
      .map(|tag| tag.name().cyan().to_string())
      .collect();

    let tags_str = tags.join(", ");
    let project_name = self.get_project_name(task.project_id());
    let mut project_name_msg = project_name.as_str().red();
    if task.is_paused() {
      project_name_msg = (project_name + " [paused]").yellow();
    }

    let time_frames = get_formatted_time_intervals(task);
    println!(
      "{line_indent}{task_id}{pad}{time_frame}{pad}{duration:7}{pad}{project:10}{pad}{tags}",
      line_indent = ViewPaddings::LINE_INDENT,
      pad = ViewPaddings::PAD,
      task_id = format_id(task.id()),
      time_frame = time_frames.first().unwrap(),
      duration = format_duration(task.duration()),
      project = project_name_msg,
      tags = tags_str.italic()
    );

    let mut task_description = match show_full {
      true => Some(task.title().dimmed().italic()),
      false => None,
    };

    if time_frames.len() > 1 {
      for time_frame in time_frames.iter().skip(1) {
        println!(
          "{padding}{time_frame}{task_description_padding}{description}",
          padding = ViewPaddings::TILL_TIME_FRAME,
          task_description_padding = ViewPaddings::FROM_DURATION_TILL_DESCRIPTION,
          description = task_description.take().unwrap_or_default()
        );
      }
    } else if task_description.is_some() {
      println!(
        "{padding}{description}",
        padding = ViewPaddings::TILL_PROJECT,
        description = task_description.take().unwrap_or_default()
      );
    }
  }
}

pub fn format_id(id: uuid::Uuid) -> String {
  let id_string = id.as_simple().to_string();
  format!(
    "{}..{}",
    &id_string[0..4],
    &id_string[id_string.len() - 4..id_string.len()]
  )
}

fn get_formatted_time_intervals(task: &Task) -> Vec<String> {
  let interval_count = task.times().len();
  let mut formatted_time_frames = Vec::new();
  for i in 0..interval_count {
    let time_frame = task.times()[i].clone();

    let is_first = i == 0;
    let is_last = i == interval_count - 1;

    let mut start_color = Color::Green;
    let mut stop_color = Color::Green;
    if !is_first && !is_last {
      start_color = Color::Magenta;
      stop_color = Color::Magenta;
    } else if is_last {
      if !is_first {
        start_color = Color::Magenta;
      }
      if task.is_paused() {
        stop_color = Color::Red;
      }
    } else if is_first {
      stop_color = Color::Magenta;
    }

    formatted_time_frames.push(format_time_frame(
      &time_frame,
      start_color,
      match time_frame.stop_time.is_some() {
        true => stop_color,
        false => Color::Yellow,
      },
    ));
  }
  return formatted_time_frames;
}

fn format_time_frame(
  time_interval: &DateTimeInterval,
  start_time_color: Color,
  stop_time_color: Color,
) -> String {
  format!(
    "{start_time} to {stop_time}",
    start_time = format_time(&time_interval.start_time, start_time_color),
    stop_time = format_time(
      &time_interval.stop_time.unwrap_or(chrono::Local::now()),
      stop_time_color
    )
  )
}

fn format_time(time: &chrono::DateTime<chrono::Local>, color: Color) -> ColoredString {
  return time.naive_local().format("%H:%M").to_string().color(color);
}
