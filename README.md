# Rune

**Rune** – *Readable Unified Notation for Everyone* – is a modern, simple, and memory-safe configuration language for Rust projects. Inspired by Markdown and classic DSL simplicity, Rune makes writing, reading, and parsing config files a breeze, while supporting advanced features like references, imports, and environment variable expansion.

> ⚠️ **Work In Progress:** Some features like `$sys` (query system parameters) and `$runtime` (query Rune runtime info) are planned and coming soon as well as simple `if` statements.

---

## Why Rune?

Rune isn’t just another config format — it’s designed to combine **readability, safety, and power**:

- **Readable & Minimal** – inspired by Markdown, so configs are easy to read for humans.
- **Safe & Reliable** – written in Rust, memory-safe, and zero external dependencies.
- **Flexible** – supports imports, variable references, arrays, nested objects, and environment expansion.
- **Future-ready** – planned `$sys` and `$runtime` namespaces plus conditional logic make it more than a static config.
- **Interoperable** – built-in Serde integration lets you export configs to JSON effortlessly.


## Planned Features

- **$sys namespace** – access system parameters.
- **$runtime namespace** – query runtime information from Rune itself.
- **Conditional logic** – simple `if` statements.

---

## Getting Started

Add `rune_cfg` to your `Cargo.toml`:

```toml
[dependencies]
rune_cfg = "0.1.0"
```


## Example `.rune` File

```rune
@description "Simple app using Rune config"
name "RuneApp"

app:
  name name
  version "1.0.0"
  debug true

  server:
    host defaults.server.host
    port 8080
    timeout "30s"
  end

  plugins [
    "auth"
    "logger"
  ]
end
```

## Parsing & Exporting to JSON
```
use rune_cfg::export_rune_file;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let json_output = export_rune_file("examples/example.rune")?;
    println!("{}", json_output);
    Ok(())
}
```

### Sample json_output

```
{
  "globals": {
    "name": "RuneApp"
  },
  "items": {
    "app": {
      "name": "name",
      "version": "1.0.0",
      "debug": true,
      "server": {
        "host": "defaults.server.host",
        "port": 8080,
        "timeout": "30s"
      },
      "plugins": ["auth", "logger"]
    }
  },
  "metadata": {
    "description": "Simple app using Rune config"
  }
}
```

