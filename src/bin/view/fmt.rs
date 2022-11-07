pub fn format_duration(duration: chrono::Duration) -> String {
  let hours = duration.num_hours();
  let minutes = duration.num_minutes();

  return format!(
    "{hours:>3} {minutes:>3}",
    hours = format_number(hours, "h"),
    minutes = format_number_force(minutes % 60, "m")
  );
}

pub fn format_duration_without_paddings(duration: chrono::Duration) -> String {
  let hours = duration.num_hours();
  let minutes = duration.num_minutes();

  return format!(
    "{hours}{pad}{minutes:>3}",
    hours = format_number_without_paddings(hours, "h"),
    pad = match hours == 0 {
      true => "",
      false => " ",
    },
    minutes = format_number_force(minutes % 60, "m"),
  );
}

fn format_number(number: i64, prefix: &str) -> String {
  match number == 0 {
    true => String::new(),
    false => format_number_force(number, prefix),
  }
}

fn format_number_without_paddings(number: i64, prefix: &str) -> String {
  let is_zero_and_not_minutes = number == 0 && prefix != "m";
  match is_zero_and_not_minutes {
    true => String::new(),
    false => format!("{number}{prefix}"),
  }
}

fn format_number_force(number: i64, prefix: &str) -> String {
  match prefix {
    "h" => format!("{:2}{prefix}", number),
    _ => format!("{:02}{prefix}", number),
  }
}
