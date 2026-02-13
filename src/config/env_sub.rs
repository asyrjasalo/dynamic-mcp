use crate::config::schema::McpServerConfig;
use regex::Regex;
use std::collections::HashMap;

pub fn substitute_env_vars(value: &str) -> String {
    let pattern = Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}").unwrap();

    pattern
        .replace_all(value, |caps: &regex::Captures| {
            let var_name = &caps[1];
            match std::env::var(var_name) {
                Ok(val) => val,
                Err(_) => {
                    tracing::warn!(
                        "Environment variable '{}' not defined, keeping placeholder",
                        var_name
                    );
                    caps[0].to_string()
                }
            }
        })
        .to_string()
}

pub fn substitute_in_object(obj: HashMap<String, String>) -> HashMap<String, String> {
    obj.into_iter()
        .map(|(k, v)| (k, substitute_env_vars(&v)))
        .collect()
}

pub fn substitute_in_array(arr: Vec<String>) -> Vec<String> {
    arr.into_iter().map(|s| substitute_env_vars(&s)).collect()
}

pub fn substitute_in_config(config: McpServerConfig) -> McpServerConfig {
    match config {
        McpServerConfig::Stdio {
            description,
            command,
            args,
            env,
            features,
            enabled,
            timeout,
        } => McpServerConfig::Stdio {
            description,
            command,
            args: args.map(substitute_in_array),
            env: env.map(substitute_in_object),
            features,
            enabled,
            timeout,
        },
        McpServerConfig::Http {
            description,
            url,
            headers,
            oauth_client_id,
            oauth_scopes,
            features,
            enabled,
            timeout,
        } => McpServerConfig::Http {
            description,
            url: substitute_env_vars(&url),
            headers: headers.map(substitute_in_object),
            oauth_client_id: oauth_client_id.map(|id| substitute_env_vars(&id)),
            oauth_scopes,
            features,
            enabled,
            timeout,
        },
        McpServerConfig::Sse {
            description,
            url,
            headers,
            oauth_client_id,
            oauth_scopes,
            features,
            enabled,
            timeout,
        } => McpServerConfig::Sse {
            description,
            url: substitute_env_vars(&url),
            headers: headers.map(substitute_in_object),
            oauth_client_id: oauth_client_id.map(|id| substitute_env_vars(&id)),
            oauth_scopes,
            features,
            enabled,
            timeout,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_substitute_env_vars_with_braces() {
        std::env::set_var("TEST_VAR_BRACES", "hello");
        assert_eq!(substitute_env_vars("${TEST_VAR_BRACES}"), "hello");
        assert_eq!(
            substitute_env_vars("prefix_${TEST_VAR_BRACES}_suffix"),
            "prefix_hello_suffix"
        );
        std::env::remove_var("TEST_VAR_BRACES");
    }

    #[test]
    fn test_substitute_env_vars_without_braces() {
        std::env::set_var("TEST_VAR_NO_BRACES", "hello");
        assert_eq!(
            substitute_env_vars("$TEST_VAR_NO_BRACES"),
            "$TEST_VAR_NO_BRACES"
        );
        std::env::remove_var("TEST_VAR_NO_BRACES");
    }

    #[test]
    fn test_substitute_env_vars_undefined() {
        let result = substitute_env_vars("${UNDEFINED_VAR}");
        assert_eq!(result, "${UNDEFINED_VAR}");
    }

    #[test]
    fn test_substitute_in_array() {
        std::env::set_var("TEST_HOME_VAR", "/home/user");
        let arr = vec!["${TEST_HOME_VAR}/.config".to_string(), "test".to_string()];
        let result = substitute_in_array(arr);
        assert_eq!(result, vec!["/home/user/.config", "test"]);
        std::env::remove_var("TEST_HOME_VAR");
    }
}
