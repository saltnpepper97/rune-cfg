//! # rune-cfg
//!
//! A minimal, powerful configuration language with native regex support.
//!
//! ## Features
//!
//! - **Clean syntax** - Minimal and readable, no complex indentation rules
//! - **Native regex** - First-class regex support with `r"pattern"` syntax
//! - **Conditionals** - Simple `if/else` for dynamic configurations
//! - **Type safety** - Strong typing with comprehensive error messages
//! - **System integration** - Access environment variables with `$env` and system info with `$sys`
//! - **Flexible keys** - Automatic `snake_case` and `kebab-case` handling
//!
//! ## Quick Example
//!
//! ```rune
//! # config.rune
//! @author "Your Name"
//! @description "Application config"
//!
//! environment "production"
//! debug false
//!
//! database:
//!   host if environment = "production" "prod.db.com" else "localhost"
//!   port 5432
//!   timeout 30
//! end
//!
//! allowed_ips [
//!   r"192\.168\.1\.\d+"
//!   r"10\.0\.0\.\d+"
//! ]
//!
//! server:
//!   user $env.USER
//!   home $env.HOME
//!   hostname $sys.hostname
//!   cpu_count $sys.cpu_count
//! end
//! ```
//!
//! ## Usage
//!
//! ```no_run
//! use rune_cfg::RuneConfig;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let config = RuneConfig::from_file("config.rune")?;
//!
//! let db_host: String = config.get("database.host")?;
//! let db_port: u16 = config.get("database.port")?;
//! let debug: bool = config.get("debug")?;
//!
//! let allowed_ips = config.get_value("allowed_ips")?;
//! for ip in &["192.168.1.100", "10.0.0.50", "172.16.0.1"] {
//!     if allowed_ips.matches(ip) {
//!         println!("{} is allowed", ip);
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Syntax Guide
//!
//! ### Basic Values
//! ```rune
//! name "value"
//! port 8080
//! enabled true
//! timeout 30.5
//! nothing null
//! ```
//!
//! ### Objects
//! ```rune
//! server:
//!   host "localhost"
//!   port 8080
//! end
//! ```
//!
//! ### Arrays
//! ```rune
//! ports [80, 443, 8080]
//! hosts ["localhost", "example.com"]
//! ```
//!
//! ### Regex Patterns
//! ```rune
//! email_pattern r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$"
//! ip_pattern r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}"
//! ```
//!
//! ### Conditionals
//! ```rune
//! environment "production"
//! db_host if environment = "production" "prod.db.com" else "localhost"
//! workers if sys.cpu_count = "8" 8 else 4
//! ```
//!
//! ### Environment Variables
//! ```rune
//! user $env.USER
//! home $env.HOME
//! path $env.PATH
//! ```
//!
//! ### System Information
//! ```rune
//! os $sys.os
//! hostname $sys.hostname
//! cpu_count $sys.cpu_count
//! memory_total $sys.memory_total
//! kernel_version $sys.kernel_version
//! ```
//!
//! ### Metadata
//! ```rune
//! @author "Your Name"
//! @version "1.0.0"
//! @description "Config file"
//! ```
//!
//! ### Imports
//! ```rune
//! gather "defaults.rune" as defaults
//!
//! server:
//!   host defaults.server.host
//!   port defaults.server.port
//! end
//! ```
//!
//! ## Error Handling
//!
//! All errors include line numbers and helpful hints:
//!
//! ```text
//! Error: Invalid regex pattern: unclosed character class
//!   → Line 15: pattern r"[a-z"
//! Hint: Check your regex syntax
//! ```

pub mod ast;
pub mod error;
pub mod export;
pub mod lexer;
pub mod parser;
pub mod resolver;
pub mod utils;
pub mod config;

pub use ast::{Document, Value};
pub use error::RuneError;
pub use config::RuneConfig;
