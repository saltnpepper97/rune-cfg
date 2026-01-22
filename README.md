<p align="center">
  <img src="./assets/rune.png" width="350" />
</p>
<h1 align="center">RUNE</h1>
<p align="center">
  <em>Readable Unified Notation for Everyone</em>
</p>
<p align="center">
  A minimal, powerful configuration language with native regex support
</p>

---

## Overview

RUNE is a configuration language designed to combine **readability, simplicity, and power**. Inspired by the best parts of TOML and YAML, RUNE makes writing and reading config files intuitive while supporting advanced features like native regex patterns, conditionals, and system/environment variable interpolation.

**Key Features:**
- **Clean syntax** - Minimal and readable, no complex indentation rules
- **Native regex** - First-class regex support with `r"pattern"` syntax
- **Conditionals** - Inline `if/else` expressions *and* block-style `if / else / endif`
- **Memory-safe** - Written in Rust with strong type safety
- **System integration** - Access environment variables with `$env` and system info with `$sys`
- **Flexible keys** - Automatic `snake_case` and `kebab-case` handling
- **Import system** - Modular configs with `gather "file.rune" as alias`
- **Type safety** - Strong typing with comprehensive error messages

## Installation

Add `rune-cfg` to your `Cargo.toml`:

```toml
[dependencies]
rune-cfg = "0.4.0"
```

## Quick Example

**config.rune:**
```rune
@description "Web server configuration"

environment "production"
app_name "MyWebServer"
default_port 8080

server:
  name app_name
  host if environment = "production" "prod.example.com" else "localhost"
  port if environment = "production" 443 else default_port
  
  database:
    url $env.DATABASE_URL
    timeout "30s"
    max_connections if $sys.cpu_count = "8" 100 else 50
    
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
use rune_cfg::RuneConfig;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = RuneConfig::from_file("config.rune")?;
    
    let host: String = config.get("server.host")?;
    let port: u16 = config.get("server.port")?;
    let max_conn: u32 = config.get("server.database.max_connections")?;
    
    println!("Connecting to {}:{}", host, port);
    println!("Max connections: {}", max_conn);
    
    Ok(())
}
```

## Syntax Highlights

### Basic Types

```rune
name "RUNE Config"
path '/usr/local/bin'

port 8080
timeout 30.5

debug true
production false

connection_pool null
fallback_server None

servers ["web1", "web2", "web3"]
ports [8080, 8081, 8082]
```

### Native Regex Patterns

RUNE has first-class regex support. Use the `r""` syntax for regex patterns:

```rune
email_regex r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$"
log_file_pattern r".*\.log$"
config_pattern r".*\.(json|yaml|toml)$"
api_endpoint_regex r"^https?://[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}(/.*)?$"
password_policy r"^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[@$!%*?&])[A-Za-z\d@$!%*?&]{8,}$"

allowed_ips [
  r"192\.168\.1\.\d+"
  r"10\.0\.0\.\d+"
]
```

**In Rust:**
```rust
use rune_cfg::RuneConfig;

let config = RuneConfig::from_file("config.rune")?;
let allowed_ips = config.get_value("allowed_ips")?;

for ip in &["192.168.1.100", "10.0.0.50", "172.16.0.1"] {
    if allowed_ips.matches(ip) {
        println!("{} is allowed", ip);
    }
}
```

### Conditionals

RUNE supports **two kinds of conditionals**:

#### 1. Simple inline `if/else` statements:

```rune
environment "production"
debug_mode false

database_host if environment = "production" "prod.db.com" else "localhost"
database_port if environment = "production" 5432 else 5433

workers if sys.cpu_count = "8" 8 else 4
log_level if debug_mode "debug" else "info"

feature_flags:
  analytics if environment = "production" true else false
  debug_panel if debug_mode true else false
end
```

#### 2. Block conditionals (inside objects)

```rune
For more complex configuration, RUNE supports **block-style conditionals**
inside object blocks using `if / else / endif`

app:
  name "MyApp"

  if environment = "production":
    debug false
    workers 8
  else:
    debug true
    workers 2
  endif
end
```

### Variable References

```rune
app_name "MyApp"
default_timeout 30

server:
  name app_name
  timeout default_timeout
end
```

### Environment Variables

```rune
database_url $env.DATABASE_URL
home_dir $env.HOME
api_key $env.API_KEY
user $env.USER

config_path "$env.HOME/.config/myapp"
```

### System Information

```rune
hostname $sys.hostname
os_name $sys.os
kernel $sys.kernel_version
cpu_count $sys.cpu_count
memory_total $sys.memory_total
uptime $sys.uptime

max_workers if sys.cpu_count = "8" 16 else 8
```

Available `$sys` keys:
- `os` - Operating system name
- `hostname` - System hostname
- `kernel_version` - Kernel version
- `os_version` - OS version
- `cpu_arch` - CPU architecture
- `cpu_count` - Number of CPU cores
- `memory_total` - Total system memory
- `memory_free` - Free memory
- `memory_used` - Used memory
- `uptime` - System uptime
- `product_name` - Product name

### Imports

```rune
gather "database.rune" as db
gather "logging.rune" as log

server:
  db_host db.host
  db_port db.port
  log_level log.level
end
```

### Comments and Metadata

```rune
# This is a comment
@description "Application configuration"
@version "1.0.0" 
@author "Your Name"

server:
  port 8080  # Default HTTP port
end
```

### Objects and Nesting

```rune
server:
  host "localhost"
  port 8080
  
  ssl:
    enabled true
    cert_path "/etc/ssl/cert.pem"
    key_path "/etc/ssl/key.pem"
  end
  
  timeouts:
    read 30
    write 30
    idle 120
  end
end
```

## Real-World Example

Here's a configuration from [Stasis](https://github.com/your-username/stasis), a Wayland idle manager:

```rune
@author "Dustin Pilgrim"
@description "Stasis configuration file"

stasis:
  pre_suspend_command None
  monitor_media true
  ignore_remote_media true
  respect_idle_inhibitors true
  debounce-seconds 5
  notify-on-unpause true
  
  inhibit_apps [
    "mpv"
    r"firefox.*"
    r".*\.exe"
  ]
  
  lock_screen:
    timeout 300
    command "loginctl lock-session"
    resume-command "notify-send 'Welcome Back $env.USER!'"
    lock-command "hyprlock"
    notification "Locking session in 10s"
    notify-seconds-before 10
  end 
  
  dpms:
    timeout 360
    command "hyprctl dispatch dpms off"
    resume-command "hyprctl dispatch dpms on"
  end
  
  suspend:
    timeout 1800
    command "systemctl suspend"
  end
end

profiles:
  gaming:
    inhibit_apps [
      r".*\.exe"
      r"steam_app_.*"
    ]
  end
end
```

## Rust API

### Loading Configuration

```rust
use rune_cfg::RuneConfig;

let config = RuneConfig::from_file("config.rune")?;

let host: String = config.get("server.host")?;
let port: u16 = config.get("server.port")?;
let debug: bool = config.get("debug")?;

let timeout = config.get_or("server.timeout", 30u64);

if let Ok(Some(api_key)) = config.get_optional::<String>("api.key") {
    println!("API key configured: {}", api_key);
}
```

### Pattern Matching

```rust
use rune_cfg::RuneConfig;

let config = RuneConfig::from_file("config.rune")?;
let patterns = config.get_value("inhibit_apps")?;

let app_id = "firefox.desktop";
if patterns.matches(app_id) {
    println!("{} matches!", app_id);
}
```

### Export to JSON

```rust
use rune_cfg::export_rune_file;

let json = export_rune_file("config.rune")?;
println!("{}", json);
```

## Error Messages

RUNE provides clear, helpful error messages with line numbers:

```text
Error: Invalid regex pattern: unclosed character class
  → Line 15: pattern r"[a-z"
Hint: Check your regex syntax
```

## Status

RUNE is production-ready and actively maintained. All core features are stable and tested.

## License

[MIT License](LICENSE)
