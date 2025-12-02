use super::*;

impl RuneConfig {
    /// Get a value with validation - returns detailed error with line info if validation fails
    pub fn get_validated<T, F>(&self, path: &str, validator: F, valid_values: &str) -> Result<T, RuneError> 
    where
        T: TryFrom<Value, Error = RuneError>,
        F: FnOnce(&T) -> bool,
    {
        let value = self.get_value(path)?;
        let typed_value = T::try_from(value)?;
        
        if !validator(&typed_value) {
            let (line, snippet) = helpers::find_config_line(path, &self.raw_content);
            return Err(RuneError::ValidationError {
                message: format!(
                    "Invalid value for `{}`\nExpected: {}",
                    path, valid_values
                ),
                line,
                column: 0,
                hint: Some(format!("Valid values are: {}\n  → {}", valid_values, snippet)),
                code: Some(450),
            });
        }
        
        Ok(typed_value)
    }

    /// Get a string value and validate it's one of the allowed values
    pub fn get_string_enum(&self, path: &str, allowed_values: &[&str]) -> Result<String, RuneError> {
        let value: String = self.get(path)?;
        let lower_value = value.to_lowercase();
        
        if !allowed_values.iter().any(|&v| v.to_lowercase() == lower_value) {
            let (line, snippet) = helpers::find_config_line(path, &self.raw_content);
            return Err(RuneError::ValidationError {
                message: format!(
                    "Invalid value '{}' for `{}`",
                    value, path
                ),
                line,
                column: 0,
                hint: Some(format!("Expected one of: {}\n  → {}", allowed_values.join(", "), snippet)),
                code: Some(451),
            });
        }
        
        Ok(value)
    }

    /// Check if a path exists in the raw content (for better error reporting)
    pub fn path_exists_in_content(&self, path: &str) -> bool {
        let (line, _) = helpers::find_config_line(path, &self.raw_content);
        line > 0
    }
}
