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
- **Flexible data types** - Strings, numbers, booleans, arrays, and nested objects
- **Variable references** - Reference global variables and imported values
- **Environment integration** - Access environment variables with `$env.VARIABLE`
- **Import system** - Modular configs with `gather "file.rune" as alias`
- **Serde integration** - Export to JSON seamlessly

## Installation

Add `rune-cfg` to your `Cargo.toml`:

```toml
[dependencies]
rune-cfg = "0.1.0"
```

## Quick Example

**config.rune:**
```rune
@description "Web server configuration"

# Global variables
app_name "MyWebServer"
default_port 8080

# Main configuration block
server:
  name app_name
  port default_port
  host $env.HOST
  
  database:
    url $env.DATABASE_URL
    timeout "30s"
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
    "default_port": 8080
  },
  "items": {
    "server": {
      "name": "MyWebServer",
      "port": 8080,
      "host": "localhost",
      "database": {
        "url": "postgresql://...",
        "timeout": "30s"
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

# Arrays
servers ["web1", "web2", "web3"]
ports [8080, 8081, 8082]
```

### Variable References
```rune
# Global variable
app_name "MyApp"

# Use in other places
server:
  name app_name  # References the global variable
end
```

### Environment Variables
```rune
# Access environment variables
database_url $env.DATABASE_URL
home_dir $env.HOME
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
end
```

### Raw Strings (for Regex)
```rune
# Raw strings preserve exact content
email_pattern r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$"
path_matcher r".*\.exe$"
```

### Comments and Metadata
```rune
# This is a comment

@description "Application configuration"
@version "1.0.0"
@author "Your Name"

# Your config here...
```

## Coming Soon

The following features are planned for future releases:

- **`$sys` namespace** - Access system information (OS, architecture, etc.)
- **`$runtime` namespace** - Query RUNE runtime information  
- **Conditional logic** - Simple `if` statements for dynamic configs

## Status

RUNE is currently in active development. The core features are stable and ready for use, but some advanced features are still being implemented.

## License

[MIT License](LICENSE)
