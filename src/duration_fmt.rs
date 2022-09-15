pub fn format_duration(duration: chrono::Duration) -> String {
  let hours = duration.num_hours();
  let minutes = duration.num_minutes();

  return format!(
    "{} {}",
    format_number(hours, "h"),
    format_number_force(minutes % 60, "m")
  );
}

pub fn format_duration_without_paddings(duration: chrono::Duration) -> String {
  let hours = duration.num_hours();
  let minutes = duration.num_minutes();

  return format!(
    "{}{}",
    format_number_without_paddings(hours, "h"),
    format_number_without_paddings(minutes % 60, "m"),
  );
}

fn format_number(number: i64, prefix: &str) -> String {
  match number == 0 {
    true => "".to_owned(),
    false => format_number_force(number, prefix),
  }
}

fn format_number_without_paddings(number: i64, prefix: &str) -> String {
  let is_zero_and_not_minutes = number == 0 && prefix != "s";
  match is_zero_and_not_minutes {
    true => "".to_owned(),
    false => format!("{}{} ", number, prefix),
  }
}

fn format_number_force(number: i64, prefix: &str) -> String {
  match prefix {
    "h" => format!("{:2}{}", number, prefix),
    _ => format!("{:02}{}", number, prefix),
  }
}
