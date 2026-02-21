use regex_lite::Regex;
use std::env;
use toml::Value;

pub struct Interpolator;

impl Interpolator {
    pub fn interpolate(content: &str) -> Result<String, String> {
        let env_expanded = Self::interpolate_env_variables(content)?;
        Self::interpolate_files(&env_expanded)
    }

    fn interpolate_env_variables(content: &str) -> Result<String, String> {
        let mut result = content.to_string();

        // Matches ${VAR:default}
        let env_var_with_fallback_re = Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*):([^}]*)\}")
            .expect("Failed to compile ENV_VAR_WITH_FALLBACK_RE");

        for caps in env_var_with_fallback_re.captures_iter(content) {
            let default_value = &caps[2];
            let replacement = env::var(&caps[1]).unwrap_or_else(|_| default_value.to_string());

            result = result.replace(&caps[0], &replacement);
        }

        let content = result.clone();

        // Matches ${VAR} — sin fallback, falla si no existe
        let env_var_braced_re = Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}")
            .expect("Failed to compile ENV_VAR_BRACED_RE");

        for caps in env_var_braced_re.captures_iter(&content) {
            let var_name = &caps[1];

            let val = env::var(var_name)
                .map_err(|_| format!("environment variable '{var_name}' not found"))?;

            result = result.replace(&caps[0], &val);
        }

        Ok(result)
    }

    fn interpolate_files(content: &str) -> Result<String, String> {
        let mut result = content.to_string();

        // Matches file:/path/to/file:default_value
        let file_with_fallback_re = Regex::new(r#"file:([^:"'\s]+):([^:\s"'`\]\)]+)"#)
            .expect("Failed to compile FILE_WITH_FALLBACK_RE");

        for caps in file_with_fallback_re.captures_iter(content) {
            let default_value = caps[2].to_string();

            let replacement = std::fs::read_to_string(&caps[1])
                .map(|c| Self::escape_toml_string(&c))
                .unwrap_or(default_value);

            result = result.replace(&caps[0], &replacement);
        }

        let content = result.clone();

        // Matches file:/path/to/file
        let file_simple_re =
            Regex::new(r#"file:([^:\s"'`\]\)]+)"#).expect("Failed to compile FILE_SIMPLE_RE");

        for caps in file_simple_re.captures_iter(&content) {
            let file_path = caps[1].to_string();

            let replacement = std::fs::read_to_string(&file_path)
                .map(|c| Self::escape_toml_string(&c))
                .map_err(|e| format!("Failed to read file '{file_path}': {e}"))?;

            result = result.replace(&caps[0], &replacement);
        }

        Ok(result)
    }

    fn escape_toml_string(s: &str) -> String {
        let val = Value::String(s.to_string());
        let toml_str = val.to_string();

        let unquoted = toml_str.strip_prefix('"').and_then(|s| s.strip_suffix('"'));

        match unquoted {
            Some(inner) => inner.to_string(),
            None => toml_str,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_env_with_fallback() {
        unsafe { std::env::set_var("MY_VAR", "hello") };
        let result = Interpolator::interpolate("value: ${MY_VAR:fallback}").unwrap();
        assert_eq!(result, "value: hello");
    }

    #[test]
    fn test_env_fallback_used() {
        unsafe { std::env::remove_var("MISSING_VAR") };
        let result = Interpolator::interpolate("value: ${MISSING_VAR:default}").unwrap();
        assert_eq!(result, "value: default");
    }

    #[test]
    fn test_env_braced_exists() {
        unsafe { std::env::set_var("BRACED_VAR", "world") };
        let result = Interpolator::interpolate("hello ${BRACED_VAR}").unwrap();
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_env_braced_missing_errors() {
        unsafe { std::env::remove_var("MISSING_VAR") };
        let result = Interpolator::interpolate("${MISSING_VAR}");
        assert!(result.is_err());
    }

    #[test]
    fn test_dollar_without_braces_is_literal() {
        // $VAR ya no se interpola — debe quedar tal cual
        unsafe { std::env::set_var("PLAIN_VAR", "should_not_appear") };
        let result = Interpolator::interpolate("hello $PLAIN_VAR").unwrap();
        assert_eq!(result, "hello $PLAIN_VAR");
    }

    #[test]
    fn test_file_with_fallback() {
        let result = Interpolator::interpolate("data: file:/no/existe:mi_default").unwrap();
        assert_eq!(result, "data: mi_default");
    }

    #[test]
    fn test_file_simple() {
        let mut tmp = NamedTempFile::new().unwrap();
        write!(tmp, "contenido").unwrap();
        let path = tmp.path().to_str().unwrap();
        let result = Interpolator::interpolate(&format!("data: file:{path}")).unwrap();
        assert_eq!(result, "data: contenido");
    }

    #[test]
    fn test_file_missing_errors() {
        let result = Interpolator::interpolate("file:/ruta/inexistente");
        assert!(result.is_err());
    }
}
