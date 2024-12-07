use std::fs;

pub fn check_log_file(log_path: &str, expected: &[&str]) -> bool {
    let log_content = fs::read_to_string(log_path).expect("Failed to read log file");
    expected.iter().all(|&line| log_content.contains(line))
}
