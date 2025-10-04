pub fn format_uptime(seconds: u64) -> String {
    if seconds < 60 {
        format!("{} sec{}", seconds, if seconds != 1 { "s" } else { "" })
    } else if seconds < 3600 {
        let minutes = seconds / 60;
        format!("{} min{}", minutes, if minutes != 1 { "s" } else { "" })
    } else {
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        format!("{} hr{}, {} min{}", 
            hours, if hours != 1 { "s" } else { "" },
            minutes, if minutes != 1 { "s" } else { "" }
        )
    }
}

pub fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    const TB: f64 = GB * 1024.0;

    let b = bytes as f64;

    if b >= TB {
        format!("{:.2} TB", b / TB)
    } else if b >= GB {
        format!("{:.2} GB", b / GB)
    } else if b >= MB {
        format!("{:.2} MB", b / MB)
    } else if b >= KB {
        format!("{:.2} KB", b / KB)
    } else {
        format!("{} B", bytes)
    }
}
