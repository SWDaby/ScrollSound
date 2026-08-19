
pub fn is_enabled(value: Option<&str>, exe_path: &str) -> bool {
    value.is_some_and(|v| v == exe_path)
}
