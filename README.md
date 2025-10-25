<p align="center">
  <img src="./assets/rune.png" width="350" />
</p>
<h1 align="center">RUNE</h1>
<p align="center">
  <em>Readable Unified Notation for Everyone</em>
</p>
<p align="center">
  A modern, simple, and memory-safe configuration language for Rust projects
</p>

---

## Overview

RUNE is a configuration language designed to combine **readability, safety, and power**. Inspired by Markdown's simplicity, RUNE makes writing and reading config files intuitive while supporting advanced features like variable references, imports, and environment variable expansion.

**Key Features:**
- **Human-readable syntax** - Clean and minimal, inspired by Markdown
- **Memory-safe** - Written in Rust with zero external dependencies  
- **Flexible data types** - Strings, numbers, booleans, arrays, nested objects, and null values
- **Built-in regex parsing** - Native regex recognition with `r""` syntax for seamless pattern matching
- **Variable references** - Reference global variables and imported values
- **Environment integration** - Access environment variables with `$env.VARIABLE`
- **Import system** - Modular configs with `gather "file.rune" as alias`
- **Serde integration** - Export to JSON seamlessly

## Installation

Add `rune-cfg` to your `Cargo.toml`:

```toml
[dependencies]
rune-cfg = "0.1.33"
```

## Quick Example

**config.rune:**
```rune
@description "Web server configuration"

# Global variables
app_name "MyWebServer"
default_port 8080
db_connection null  # Will be set via environment

# Main configuration block
server:
  name app_name
  port default_port
  host $env.HOST
  
  database:
    url $env.DATABASE_URL
    connection_pool db_connection
    timeout "30s"
    
    # Built-in regex validation patterns
    email_validator r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$"
    username_pattern r"^[a-zA-Z0-9_]{3,20}$"
  end
  
  features [
    "auth"
    "logging" 
    "metrics"
  ]
end
```

**Rust code:**
```rust
use rune_cfg::export_rune_file;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let json = export_rune_file("config.rune")?;
    println!("{}", json);
    Ok(())
}
```

**Output:**
```json
{
  "globals": {
    "app_name": "MyWebServer",
    "default_port": 8080,
    "db_connection": null
  },
  "items": {
    "server": {
      "name": "MyWebServer",
      "port": 8080,
      "host": "localhost",
      "database": {
        "url": "postgresql://...",
        "connection_pool": null,
        "timeout": "30s",
        "email_validator": "^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$",
        "username_pattern": "^[a-zA-Z0-9_]{3,20}$"
      },
      "features": ["auth", "logging", "metrics"]
    }
  },
  "metadata": {
    "description": "Web server configuration"
  }
}
```

## Syntax Highlights

### Basic Types

```rune
# Strings (single or double quotes)
name "RUNE Config"
path '/usr/local/bin'

# Numbers
port 8080
timeout 30.5

# Booleans
debug true
production false

# Null values
connection_pool null
fallback_server None

# Arrays
servers ["web1", "web2", "web3"]
ports [8080, 8081, 8082]
```

### Built-in Regex Parsing

RUNE now includes native regex parsing capabilities. The `r""` syntax serves dual purposes as both raw strings and regex patterns:

```rune
# Email validation pattern
email_regex r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$"

# File path matching
log_file_pattern r".*\.log$"
config_pattern r".*\.(json|yaml|toml)$"

# URL validation
api_endpoint_regex r"^https?://[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}(/.*)?$"

# Password strength requirements
password_policy r"^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[@$!%*?&])[A-Za-z\d@$!%*?&]{8,}$"
```

> **Note:** While RUNE provides built-in regex parsing, you can still use the dedicated `regex` crate in your Rust code for advanced regex operations and performance-critical applications.

### Variable References

```rune
# Global variables
app_name "MyApp"
default_timeout null

# Use in other places
server:
  name app_name  # References the global variable
  timeout default_timeout  # Will be null
end
```

### Environment Variables

```rune
# Access environment variables
database_url $env.DATABASE_URL
home_dir $env.HOME
api_key $env.API_KEY
```

### Imports

```rune
# Import another RUNE file
gather "database.rune" as db
gather "logging.rune" as log

# Use imported values
server:
  db_host db.host
  log_level log.level
  backup_connection None  # Placeholder for future configuration
end
```

### Raw Strings & Regex Patterns

```rune
# Raw strings preserve exact content and serve as regex patterns
email_pattern r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$"
file_matcher r".*\.exe$"
json_validator r"^\{.*\}$"

# Complex regex for log parsing
log_pattern r"^\[(\d{4}-\d{2}-\d{2})\s(\d{2}:\d{2}:\d{2})\]\s(INFO|WARN|ERROR):\s(.+)$"

# IPv4 address validation
ip_regex r"^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$"
```

### Comments and Metadata

```rune
# This is a comment
@description "Application configuration"
@version "1.0.0" 
@author "Your Name"

# Your config here...
fallback_mode null  # Disabled by default
```

## Null Handling

RUNE supports explicit null values using both `null` and `None` keywords:

```rune
# Both represent null values
optional_feature null
backup_server None

# Useful for optional configuration
cache:
  redis_url $env.REDIS_URL
  fallback None
  timeout null
end
```

## Coming Soon

The following features are planned for future releases:

- **`$runtime` namespace** - Query RUNE runtime information  
- **Conditional logic** - Simple `if` statements for dynamic configs
- **Enhanced regex integration** - Built-in regex validation and matching functions

## Status

RUNE is currently in active development. The core features are stable and ready for use, but some advanced features are still being implemented.

## License

[MIT License](LICENSE)
