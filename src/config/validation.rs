use super::*;

impl RuneConfig {
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

    pub fn get_string_enum(&self, path: &str, allowed_values: &[&str]) -> Result<String, RuneError> {
        let value = self.get_value(path)?;
        
        let string_value = match value {
            Value::String(s) => s,
            _ => return Err(RuneError::TypeError {
                message: format!("Expected string for `{}`, got {:?}", path, value),
                line: 0,
                column: 0,
                hint: Some("Use a string value in your config".into()),
                code: Some(401),
            })
        };
        
        let lower_value = string_value.to_lowercase();
        
        if !allowed_values.iter().any(|&v| v.to_lowercase() == lower_value) {
            let (line, snippet) = helpers::find_config_line(path, &self.raw_content);
            return Err(RuneError::ValidationError {
                message: format!(
                    "Invalid value '{}' for `{}`",
                    string_value, path
                ),
                line,
                column: 0,
                hint: Some(format!("Expected one of: {}\n  → {}", allowed_values.join(", "), snippet)),
                code: Some(451),
            });
        }
        
        Ok(string_value)
    }

    pub fn path_exists_in_content(&self, path: &str) -> bool {
        let (line, _) = helpers::find_config_line(path, &self.raw_content);
        line > 0
    }
}
