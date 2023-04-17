use k8s_openapi::{
    apimachinery::pkg::apis::meta::v1::Time,
    chrono::{Duration, Utc},
};

pub fn format_creation_since(time: Option<Time>) -> String {
    format_duration(Utc::now().signed_duration_since(time.unwrap().0))
}

fn format_duration(dur: Duration) -> String {
    match (
        dur.num_days(),
        dur.num_hours(),
        dur.num_minutes(),
        dur.num_seconds(),
    ) {
        (days, hours, _, _) if hours > 2 * 24 => format!("{}d{}h", days, hours - days * 24),
        (_, hours, mins, _) if mins > 2 * 60 => format!("{}h{}m", hours, mins - hours * 60),
        (_, _, mins, seconds) if seconds > 2 * 60 => format!("{}m{}s", mins, seconds - mins * 60),
        (_, _, _, seconds) => format!("{}s", seconds),
    }
}
