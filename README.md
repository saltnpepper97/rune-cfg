<p align="center">
  <img src="./assets/rune.png" width="550" />
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
- **Schemas** - Validate configs with required fields, types, enums, ranges, and arrays
- **Type safety** - Strong typing with comprehensive error messages

## Installation

Add `rune-cfg` to your `Cargo.toml`:

```toml
[dependencies]
rune-cfg = "0.4.6"
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

### Schemas

Schemas describe the expected shape of a RUNE config. They are parsed separately from runtime config files and return structured diagnostics that can be shown in CLIs, tests, and `rune-lsp`.

```rune
schema app:
  name string required
  version string required
  debug bool default false
  environment enum ["dev", "staging", "production"] required

  server:
    host string required
    port int range 1..65535 default 8080
  end

  plugins [string]
end
```

Supported schema types:
- `string`
- `int`
- `float`
- `number`
- `bool`
- `regex`
- `null`
- `any`
- `enum ["a", "b"]`
- `[type]` arrays, such as `[string]`
- nested object blocks

Supported constraints:
- `required` requires a config path to exist
- `default <value>` documents the fallback value expected by callers
- `range min..max` validates numeric values
- `enum [...]` validates string values against an allowed set

Example invalid config:

```rune
app:
  name "RuneApp"
  environment "prod"

  server:
    host "localhost"
    port "8080"
  end

  plugins ["auth", 42]
end
```

The schema validator reports diagnostics for the missing `app.version`, invalid enum value, wrong `app.server.port` type, and wrong `app.plugins[1]` array item type.

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

### Schema Validation

```rust
use rune_cfg::{RuneConfig, SchemaDocument};

let config = RuneConfig::from_file("examples/schema_config_invalid.rune")?;
let schema = SchemaDocument::from_file("examples/schema.rune")?;

for diagnostic in config.validate_schema(&schema) {
    println!("{}", diagnostic.message);
    if let Some(hint) = diagnostic.hint {
        println!("Hint: {}", hint);
    }
}
```

`validate_schema` returns `Vec<RuneDiagnostic>` instead of failing fast. This is intentional: callers can show every schema issue at once, and `rune-lsp` maps the same diagnostic shape into editor diagnostics.

### Export to JSON

```rust
use rune_cfg::export_rune_file;

let json = export_rune_file("config.rune")?;
println!("{}", json);
```

## Language Server

RUNE includes an experimental language server binary named `rune-lsp`. It speaks LSP over stdio and can be launched by any editor client that supports custom language servers.

Editor runtime files are included under `editors/nvim/` (Neovim) and `editors/vim/` (classic Vim). These provide highlighting, filetype detection, and 2-space indentation. The language server provides editor intelligence such as diagnostics, completion, hover, navigation, rename, and formatting. They are separate pieces and can be used together.

An experimental Tree-sitter grammar is included under `editors/tree-sitter-rune/` for higher quality highlighting, indentation, and folding. The Vim syntax files remain the stable fallback while the grammar matures.

Current capabilities:
- full-document sync
- syntax diagnostics for opened `.rune` files
- schema parsing diagnostics for schema files
- schema validation diagnostics for config files
- schema-aware key and enum-value completion
- hover text for schema-backed fields
- document symbols for object blocks and keys
- go-to-definition from a config key to its schema field, and from `@schema` to the schema file
- find-references for a key within a document or across schema-bound configs
- rename of a key within a document or across schema-bound configs
- document formatting (2-space block indentation)
- a quickfix code action for missing `end` diagnostics
- quickfix code actions for invalid enum values and missing required fields
- quickfix code actions for simple schema type mismatches, such as quoted numbers
- schema field descriptions from leading comments in schema files
- `@schema` name/path completion from project, user, and system schema directories
- optional `@schema` references for app-provided schemas
- automatic `schema.rune` discovery from the config file directory upward to the workspace root

Schemas are optional. `rune-lsp` supports three levels of editor intelligence:

- Plain RUNE: no schema required; syntax diagnostics still work.
- Local schema: place `schema.rune` beside a config file or in a parent directory for validation, completion, and hover.
- App-provided schema: add `@schema "name"` or `@schema "./path/to/schema.rune"` to use a named or explicit schema.

Explicit schema references are resolved before local discovery. Path references may be relative to the config file directory or absolute:

```rune
@schema "./schemas/app.rune"
@schema "../schemas/app.rune"
@schema "/usr/share/rune/schemas/app.rune"
```

Named schemas use the schema name plus `.rune` and search these locations:

```text
./schemas/<name>.rune
./.rune/schemas/<name>.rune
~/.config/rune/schemas/<name>.rune
/usr/local/share/rune/schemas/<name>.rune
/usr/share/rune/schemas/<name>.rune
```

For example:

```rune
@schema "stasis"
```

If an explicit schema reference cannot be resolved, diagnostics include the locations that were searched. Completion inside `@schema "..."` suggests discovered schema names plus common relative paths such as `./schema.rune` and `./schemas/`.

Schema comments immediately before fields become editor hover/completion documentation:

```rune
schema app:
  # Deployment environment for this application.
  environment enum ["dev", "staging", "production"] required

  # Server listener settings.
  server:
    # Public host name or IP address.
    host string required
    port int range 1..65535 default 8080
  end
end
```

Completion uses the active schema to suggest only fields that belong in the current object, filters fields already present in that object, and uses snippets for common value shapes such as booleans, arrays, enums, and object blocks. Hover text includes the schema source so app-provided schemas are visible from the editor.

Install the released server binary with:

```sh
cargo install rune-cfg --version 0.4.6
```

Or run the server directly from this repository with:

```sh
cargo run --bin rune-lsp
```

Build a reusable local development binary with:

```sh
cargo build --bin rune-lsp
```

The binary will be available at:

```text
target/debug/rune-lsp
```

Check the binary with:

```sh
rune-lsp --version
rune-lsp --help
```

Use `target/debug/rune-lsp --version` when checking a local development build.

Example Neovim setup:

Install the optional syntax files:

```sh
cp -r editors/nvim/ftdetect ~/.config/nvim/
cp -r editors/nvim/syntax ~/.config/nvim/
cp -r editors/nvim/ftplugin ~/.config/nvim/
```

Optional Tree-sitter grammar development:

```sh
cd editors/tree-sitter-rune
npm install
npm run generate
npm test
```

Then configure the LSP in Neovim 0.11+:

```lua
vim.lsp.config("rune_lsp", {
  cmd = { "rune-lsp" },
  filetypes = { "rune" },
  root_markers = { "schema.rune", ".rune", ".git" },
})

vim.lsp.enable("rune_lsp")
```

For one-off testing without a named config:

```lua
vim.lsp.start({
  name = "rune_lsp",
  cmd = { "/path/to/rune-cfg/target/debug/rune-lsp" },
  root_dir = vim.fs.root(0, { "schema.rune", ".rune", ".git" }),
})
```

Example classic Vim setup:

Install the optional syntax files:

```sh
cp -r editors/vim/ftdetect ~/.vim/
cp -r editors/vim/syntax ~/.vim/
cp -r editors/vim/ftplugin ~/.vim/
```

Then wire up the LSP through a client such as [vim-lsp](https://github.com/prabirshrestha/vim-lsp) or [ALE](https://github.com/dense-analysis/ale). See `editors/vim/README.md` for a vim-lsp example.

Place a `schema.rune` next to your config file or in a parent directory, or use `@schema` to point at an app-provided schema. When a schema is available, `rune-lsp` validates the config and uses the schema for completion and hover. Without a schema, `rune-lsp` stays in plain RUNE mode and only reports syntax-level diagnostics.

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

[GPLv3License](LICENSE)
