use regex_lite::Regex;
use std::env;

pub fn expand_env_variables(content: &str) -> Result<String, String> {
    let re = Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*):?([^}]*)\}")
        .map_err(|e| format!("Regex error: {e}"))?;

    let mut result = content.to_string();

    for caps in re.captures_iter(content) {
        let full_match = caps.get(0).unwrap().as_str();
        let var_name = caps.get(1).unwrap().as_str();
        let default_value = caps.get(2).map_or("", |m| m.as_str());

        let replacement = if let Ok(value) = env::var(var_name) {
            value
        } else {
            if default_value.is_empty() {
                return Err(format!("environment variable '{var_name}' not found"));
            }

            default_value.to_string()
        };

        result = result.replace(full_match, &replacement);
    }

    let simple_re =
        Regex::new(r"\$([A-Za-z_][A-Za-z0-9_]*)").map_err(|e| format!("Regex error: {e}"))?;

    for caps in simple_re.captures_iter(&result.clone()) {
        let full_match = caps.get(0).unwrap().as_str();
        let var_name = caps.get(1).unwrap().as_str();

        if content.contains(&format!("${{{var_name}")) {
            continue;
        }

        let Ok(replacement) = env::var(var_name) else {
            return Err(format!("environment variable '{var_name}' not found"));
        };

        result = result.replace(full_match, &replacement);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_with_braces_syntax() {
        unsafe {
            std::env::set_var("TEST_VAR", "test_value");
        }

        let result = expand_env_variables("${TEST_VAR}").expect("failed to expand");
        assert_eq!(result, "test_value");

        unsafe {
            std::env::remove_var("TEST_VAR");
        }
    }

    #[test]
    fn test_expand_with_default_value() {
        let result = expand_env_variables("${NONEXISTENT:default_val}").expect("failed to expand");
        assert_eq!(result, "default_val");
    }

    #[test]
    fn test_expand_missing_var_no_default() {
        let result = expand_env_variables("${MISSING_VAR}");
        assert!(result.is_err());
    }
}
