pub fn format_duration(duration: chrono::Duration) -> String {
  let hours = duration.num_hours();
  let minutes = duration.num_minutes();
  let seconds = duration.num_seconds();

  let minutes_formatted = match hours > 0 {
    true => format_number_force(minutes % 60, "m"),
    false => format_number(minutes % 60, "m"),
  };

  return format!(
    "{:>4}{:>4}{:>4}",
    format_number(hours, "h"),
    minutes_formatted,
    format_number_force(seconds % 60, "s"),
  );
}

pub fn format_duration_without_paddings(duration: chrono::Duration) -> String {
  let hours = duration.num_hours();
  let minutes = duration.num_minutes();
  let seconds = duration.num_seconds();

  return format!(
    "{}{}{}",
    format_number_without_paddings(hours, "h"),
    format_number_without_paddings(minutes % 60, "m"),
    format_number_without_paddings(seconds % 60, "s"),
  );
}

fn format_number(number: i64, prefix: &str) -> String {
  match number == 0 {
    true => "".to_owned(),
    false => format_number_force(number, prefix),
  }
}

fn format_number_without_paddings(number: i64, prefix: &str) -> String {
  let is_zero_and_not_seconds = number == 0 && prefix != "s";
  match is_zero_and_not_seconds {
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
