use super::*;

impl RuneConfig {
    /// Get a typed value from the configuration using dot notation.
    ///
    /// Automatically handles both `snake_case` and `kebab-case` key names.
    ///
    /// # Examples
    /// ```no_run
    /// # use rune_cfg::RuneConfig;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let config = RuneConfig::from_file("config.rune")?;
    /// let host: String = config.get("server.host")?;
    /// let port: u16 = config.get("server.port")?;
    /// let debug: bool = config.get("debug")?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    /// Returns error if path doesn't exist or value can't be converted to type T.
    pub fn get<T>(&self, path: &str) -> Result<T, RuneError> 
    where
        T: TryFrom<Value, Error = RuneError>
    {
        let value = self.get_value_flexible(path)?;
        T::try_from(value).map_err(|e| {
            enhance_error_with_line_info(e, path, &self.raw_content)
        })
    }

    /// Get an optional typed value - returns `None` if key doesn't exist.
    ///
    /// # Examples
    /// ```no_run
    /// # use rune_cfg::RuneConfig;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let config = RuneConfig::from_file("config.rune")?;
    /// if let Ok(Some(api_key)) = config.get_optional::<String>("api.key") {
    ///     println!("API key: {}", api_key);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_optional<T>(&self, path: &str) -> Result<Option<T>, RuneError> 
    where
        T: TryFrom<Value, Error = RuneError>
    {
        match self.get_value_flexible(path) {
            Ok(value) => Ok(Some(T::try_from(value)?)),
            Err(RuneError::SyntaxError { code: Some(304), .. }) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Get a value with a fallback default.
    ///
    /// # Examples
    /// ```no_run
    /// # use rune_cfg::RuneConfig;
    /// # let config = RuneConfig::from_file("config.rune").unwrap();
    /// let timeout = config.get_or("server.timeout", 30u64);
    /// let debug = config.get_or("debug", false);
    /// ```
    pub fn get_or<T>(&self, path: &str, default: T) -> T 
    where
        T: TryFrom<Value, Error = RuneError>
    {
        self.get(path).unwrap_or(default)
    }

    /// Internal method that tries both snake_case and kebab-case variants.
    ///
    /// Allows flexible key access: `monitor_media` and `monitor-media` both work.        
    fn get_value_flexible(&self, path: &str) -> Result<Value, RuneError> {
        // Fast path: exact
        if let Ok(v) = self.get_value(path) {
            return Ok(v);
        }

        // Root path special case handled by get_value("") already
        if path.trim().is_empty() {
            return self.get_value(path);
        }

        // Try segment-by-segment variants: for each segment, try {original, snake, kebab}
        let segs: Vec<&str> = path.split('.').collect();

        fn variants(seg: &str) -> Vec<String> {
            let mut out = Vec::new();
            out.push(seg.to_string());

            let snake = seg.replace('-', "_");
            if snake != seg {
                out.push(snake.clone());
            }

            let kebab = seg.replace('_', "-");
            if kebab != seg {
                out.push(kebab);
            }

            // de-dupe
            out.sort();
            out.dedup();
            out
        }

        // DFS over combinations, stop on first that resolves
        fn dfs(
            cfg: &RuneConfig,
            segs: &[&str],
            i: usize,
            cur: &mut Vec<String>,
        ) -> Result<Value, RuneError> {
            if i == segs.len() {
                let candidate = cur.join(".");
                return cfg.get_value(&candidate);
            }

            for v in variants(segs[i]) {
                cur.push(v);
                if let Ok(val) = dfs(cfg, segs, i + 1, cur) {
                    return Ok(val);
                }
                cur.pop();
            }

            Err(RuneError::SyntaxError {
                message: format!("Path '{}' not found in configuration", segs.join(".")),
                line: 0,
                column: 0,
                hint: Some("Check that the path exists in your config file".into()),
                code: Some(304),
            })
        }

        dfs(self, &segs, 0, &mut Vec::new())
    }

    /// Get a raw `Value` from the configuration.
    ///
    /// Resolves references, conditionals, and environment/system variables.
    ///
    /// # Examples
    /// ```no_run
    /// # use rune_cfg::RuneConfig;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let config = RuneConfig::from_file("config.rune")?;
    /// let value = config.get_value("inhibit_apps")?;
    /// if value.matches("firefox.desktop") {
    ///     println!("App matches pattern");
    /// }
    /// # Ok(())
    /// # }
    /// ```       
    pub fn get_value(&self, path: &str) -> Result<Value, RuneError> {
        // Root lookup: return the fully-resolved main document as a Value::Object.
        if path.trim().is_empty() {
            let main_doc = self
                .documents
                .get(&self.main_doc_key)
                .ok_or_else(|| RuneError::SyntaxError {
                    message: "No main document loaded".into(),
                    line: 0,
                    column: 0,
                    hint: None,
                    code: Some(305),
                })?;

            let mut temp_parser = parser::Parser::new("").map_err(|_| RuneError::SyntaxError {
                message: "Failed to create temporary parser".into(),
                line: 0,
                column: 0,
                hint: None,
                code: Some(303),
            })?;

            for (alias, doc) in &self.documents {
                if alias != &self.main_doc_key {
                    temp_parser.inject_import(alias.clone(), doc.clone());
                }
            }

            // Build a root Value from the document's top-level items.
            // (items are already Vec<(String, Value)> and match Value::Object)
            let root_value = Value::Object(main_doc.items.clone());

            return helpers::resolve_value_recursively(&root_value, &temp_parser, main_doc);
        }

        let path_segments: Vec<String> = path.split('.').map(|s| s.to_string()).collect();

        if let Some(main_doc) = self.documents.get(&self.main_doc_key) {
            let mut temp_parser = parser::Parser::new("").map_err(|_| RuneError::SyntaxError {
                message: "Failed to create temporary parser".into(),
                line: 0,
                column: 0,
                hint: None,
                code: Some(303),
            })?;

            for (alias, doc) in &self.documents {
                if alias != &self.main_doc_key {
                    temp_parser.inject_import(alias.clone(), doc.clone());
                }
            }

            let resolved = temp_parser
                .resolve_reference(&path_segments, main_doc)
                .ok_or_else(|| {
                    let (line, snippet) = helpers::find_config_line(path, &self.raw_content);
                    if line > 0 {
                        RuneError::SyntaxError {
                            message: format!(
                                "Path '{}' found but could not be resolved on line {}",
                                path, line
                            ),
                            line,
                            column: 0,
                            hint: Some(format!("Check the value at: {}", snippet)),
                            code: Some(304),
                        }
                    } else {
                        RuneError::SyntaxError {
                            message: format!("Path '{}' not found in configuration", path),
                            line: 0,
                            column: 0,
                            hint: Some("Check that the path exists in your config file".into()),
                            code: Some(304),
                        }
                    }
                })?;

            helpers::resolve_value_recursively(resolved, &temp_parser, main_doc)
        } else {
            Err(RuneError::SyntaxError {
                message: "No main document loaded".into(),
                line: 0,
                column: 0,
                hint: None,
                code: Some(305),
            })
        }
    }

    /// Get all keys at a given path level.
    ///
    /// # Examples
    /// ```no_run
    /// # use rune_cfg::RuneConfig;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let config = RuneConfig::from_file("config.rune")?;
    /// let keys = config.get_keys("server")?;
    /// for key in keys {
    ///     println!("server.{}", key);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_keys(&self, path: &str) -> Result<Vec<String>, RuneError> {
        let value = self.get_value(path)?;
        match value {
            Value::Object(items) => Ok(items.iter().map(|(k, _)| k.clone()).collect()),
            _ => Err(RuneError::TypeError {
                message: format!("Path '{}' is not an object", path),
                line: 0,
                column: 0,
                hint: Some("Only objects have keys".into()),
                code: Some(306),
            })
        }
    }

    /// Check if a configuration path exists.
    ///
    /// # Examples
    /// ```no_run
    /// # use rune_cfg::RuneConfig;
    /// # let config = RuneConfig::from_file("config.rune").unwrap();
    /// if config.has("server.ssl.enabled") {
    ///     println!("SSL is configured");
    /// }
    /// ```
    pub fn has(&self, path: &str) -> bool {
        self.get_value_flexible(path).is_ok()
    }
}

/// Enhance type/validation errors with line number information from config file.
fn enhance_error_with_line_info(e: RuneError, path: &str, raw_content: &str) -> RuneError {
    match e {
        RuneError::TypeError { message, hint, code, .. } => {
            let (line, snippet) = helpers::find_config_line(path, raw_content);
            if line > 0 {
                RuneError::TypeError {
                    message: format!("{}\n  → {}", message, snippet),
                    line,
                    column: 0,
                    hint,
                    code,
                }
            } else {
                RuneError::TypeError { message, line: 0, column: 0, hint, code }
            }
        }
        RuneError::ValidationError { message, hint, code, .. } => {
            let (line, snippet) = helpers::find_config_line(path, raw_content);
            if line > 0 {
                RuneError::ValidationError {
                    message: format!("{}\n  → {}", message, snippet),
                    line,
                    column: 0,
                    hint,
                    code,
                }
            } else {
                RuneError::ValidationError { message, line: 0, column: 0, hint, code }
            }
        }
        other => other,
    }
}
