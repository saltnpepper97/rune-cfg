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
        T: TryFrom<Value, Error = RuneError>,
    {
        let value = self
            .get_value_flexible(path)
            .map_err(|e| self.not_found_with_line_info(e, path))?;
        T::try_from(value).map_err(|e| enhance_error_with_line_info(e, path, &self.raw_content))
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
        T: TryFrom<Value, Error = RuneError>,
    {
        match self.get_value_flexible(path) {
            Ok(value) => Ok(Some(T::try_from(value)?)),
            Err(RuneError::SyntaxError {
                code: Some(304), ..
            }) => Ok(None),
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
        T: TryFrom<Value, Error = RuneError>,
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
    /// Build (once) and return the fully-resolved root object.
    ///
    /// References, conditionals, `$env`/`$sys`/`var`, and inline/block `if` are all resolved
    /// and flattened a single time, then cached in `self.resolved_root`. Every value lookup
    /// reuses this, turning what used to be a full-document deep-clone + recursive resolve
    /// *per key access* into a single resolve for the lifetime of the instance.
    fn resolved_root(&self) -> Result<&Value, RuneError> {
        use crate::ast::ObjectItem;

        if let Some(resolved) = self.resolved_root.get() {
            return Ok(resolved);
        }

        let main_doc =
            self.documents
                .get(&self.main_doc_key)
                .ok_or_else(|| RuneError::SyntaxError {
                    message: "No main document loaded".into(),
                    line: 0,
                    column: 0,
                    hint: None,
                    code: Some(305),
                })?;

        // Build a temporary parser and inject imports (same as before).
        let mut temp_parser =
            parser::Parser::with_options("", self.options).map_err(|_| RuneError::SyntaxError {
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

        // Build a unified "root" object that contains both globals and items as Assigns.
        // This is crucial because block `if ... endif` can hide assignments inside ObjectItem::IfBlock,
        // and resolve_reference() cannot see those until we resolve/flatten the object.
        let mut root_items: Vec<ObjectItem> = Vec::new();

        for (k, v) in &main_doc.globals {
            root_items.push(ObjectItem::Assign(k.clone(), v.clone()));
        }
        for (k, v) in &main_doc.items {
            root_items.push(ObjectItem::Assign(k.clone(), v.clone()));
        }

        // Resolve + flatten everything (references, $env/$sys, inline if, and block if/endif).
        let resolved =
            helpers::resolve_value_recursively(&Value::Object(root_items), &temp_parser, main_doc)?;

        // Cache it. If another thread raced us to set it first, that's fine — the resolution
        // is deterministic, so we just use whichever value won.
        let _ = self.resolved_root.set(resolved);
        Ok(self
            .resolved_root
            .get()
            .expect("resolved_root was just set"))
    }

    pub fn get_value(&self, path: &str) -> Result<Value, RuneError> {
        let resolved_root = self.resolved_root()?;

        // Root lookup: return fully resolved root
        if path.trim().is_empty() {
            return Ok(resolved_root.clone());
        }

        // Now traverse the resolved Value tree to find the requested path.
        fn lookup_path(v: &Value, segs: &[&str]) -> Option<Value> {
            use crate::ast::ObjectItem;

            let mut cur = v;
            for seg in segs {
                match cur {
                    Value::Object(items) => {
                        let next = items.iter().find_map(|it| match it {
                            ObjectItem::Assign(k, val) if k == seg => Some(val),
                            _ => None,
                        })?;
                        cur = next;
                    }
                    Value::Annotated(value) => {
                        cur = value
                            .attributes
                            .iter()
                            .find_map(|(key, value)| (key == seg).then_some(value))?;
                    }
                    _ => return None,
                }
            }
            Some(cur.clone())
        }

        let segs: Vec<&str> = path.split('.').collect();
        // Return a cheap "not found" error here. Deriving a line number requires scanning the
        // whole raw config, and this path is extremely hot: `get_value_flexible` retries
        // snake/kebab variants and callers probe many candidate paths, so most lookups miss
        // several times before hitting. `get_optional`/`has` discard the error entirely. Line
        // info is attached lazily at the `get()` boundary (see `not_found_with_line_info`).
        lookup_path(resolved_root, &segs).ok_or_else(|| Self::not_found_error(path))
    }

    /// Cheap "path not found" error with no raw-config scan.
    fn not_found_error(path: &str) -> RuneError {
        RuneError::SyntaxError {
            message: format!("Path '{}' not found in configuration", path),
            line: 0,
            column: 0,
            hint: Some("Check that the path exists in your config file".into()),
            code: Some(304),
        }
    }

    /// Enrich a "not found" (code 304) error with a best-effort line number by scanning the
    /// raw config. Only called on the cold user-facing `get()` failure path, never on the hot
    /// `get_optional`/`has`/variant-probe paths.
    fn not_found_with_line_info(&self, e: RuneError, path: &str) -> RuneError {
        let RuneError::SyntaxError {
            code: Some(304), ..
        } = &e
        else {
            return e;
        };
        let (line, snippet) = helpers::find_config_line(path, &self.raw_content);
        if line > 0 {
            RuneError::SyntaxError {
                message: format!(
                    "Path '{}' not found in configuration (near line {})",
                    path, line
                ),
                line,
                column: 0,
                hint: Some(format!("Check around: {}", snippet)),
                code: Some(304),
            }
        } else {
            e
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
        use crate::ast::ObjectItem;

        let value = self.get_value(path)?;
        match value {
            Value::Object(items) => Ok(items
                .iter()
                .filter_map(|it| match it {
                    ObjectItem::Assign(k, _) => Some(k.clone()),
                    _ => None,
                })
                .collect()),
            Value::Annotated(value) => Ok(value
                .attributes
                .iter()
                .map(|(key, _)| key.clone())
                .collect()),
            _ => Err(RuneError::TypeError {
                message: format!("Path '{}' is not an object", path),
                line: 0,
                column: 0,
                hint: Some("Only objects have keys".into()),
                code: Some(306),
            }),
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
        RuneError::TypeError {
            message,
            hint,
            code,
            ..
        } => {
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
                RuneError::TypeError {
                    message,
                    line: 0,
                    column: 0,
                    hint,
                    code,
                }
            }
        }
        RuneError::ValidationError {
            message,
            hint,
            code,
            ..
        } => {
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
                RuneError::ValidationError {
                    message,
                    line: 0,
                    column: 0,
                    hint,
                    code,
                }
            }
        }
        other => other,
    }
}
