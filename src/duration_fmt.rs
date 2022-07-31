pub fn format_duration(duration: chrono::Duration) -> String {
  let hours = duration.num_hours();
  let minutes = duration.num_minutes();
  let seconds = duration.num_seconds();

  return format!(
    "{:>4}{:>4}{:>4}",
    format_number(hours, "h"),
    format_number(minutes % 60, "m"),
    format_number_force(seconds % 60, "s"),
  );
}

fn format_number(number: i64, prefix: &str) -> String {
  match number == 0 {
    true => "".to_owned(),
    false => format_number_force(number, prefix),
  }
}

fn format_number_force(number: i64, prefix: &str) -> String {
  match prefix {
    "h" => format!("{:2}{}", number, prefix),
    _ => format!("{:02}{}", number, prefix),
  }
}
