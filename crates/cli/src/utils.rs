use regex::Regex;

pub fn pad_truncate(input: &str, width: usize) -> String {
    if input.len() <= width {
        format!("{input:width$}")
    } else if width <= 3 {
        input.chars().take(width).collect()
    } else {
        let trimmed: String = input.chars().take(width - 3).collect();
        format!("{trimmed}...")
    }
}

pub fn task_slug(title: &str) -> String {
    let lower = title.to_lowercase();
    let re = Regex::new(r"[^a-z0-9]+").unwrap();
    let slug = re.replace_all(&lower, "-");
    let trimmed = slug.trim_matches('-');
    let cut: String = trimmed.chars().take(16).collect();
    cut.trim_end_matches('-').to_string()
}

pub fn truncate_title(prompt: &str) -> String {
    let trimmed = prompt.trim();
    if trimmed.len() <= 60 {
        trimmed.to_string()
    } else {
        let cut: String = trimmed.chars().take(60).collect();
        format!("{cut}...")
    }
}

pub fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}
