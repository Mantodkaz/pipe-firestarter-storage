use std::env;

/// bug fixes
pub fn get_current_executable_path() -> String {
    match env::current_exe() {
        Ok(exe_path) => exe_path.to_string_lossy().to_string(),
        Err(_) => "pipe".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executable_detection() {
        let exe_path = get_current_executable_path();
        assert!(!exe_path.is_empty());
    }
}
