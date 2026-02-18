# thisconfig

TOML configuration loader with environment variable interpolation and deep merging.

## Installation

```bash
cargo add thisconfig
```

## API Reference

| Item | Type | Description |
|------|------|-------------|
| `Config` | Struct | Holds loaded configuration as a TOML table. |
| `builder()` | Method on `Config` | Returns a `ConfigBuilder` for flexible config loading. |
| `get<T: ConfigItem>()` | Method on `Config` | Retrieves and deserializes a config section, returns `Option<T>`. |
| `get_or_default<T: ConfigItem + Default>()` | Method on `Config` | Retrieves config or returns default if missing/invalid. |
| `get_or_panic<T: ConfigItem>()` | Method on `Config` | Retrieves config or panics if missing/invalid. |
| `get_validated<T: ConfigItem + Validate>()` | Method on `Config` (feature: validation) | Retrieves and validates config, returns `Result<T, ConfigError>`. |
| `ConfigBuilder` | Struct | Builder for configuring sources and building `Config`. |
| `add_file(path)` | Method on `ConfigBuilder` | Adds an optional config file source. |
| `add_required_file(path)` | Method on `ConfigBuilder` | Adds a required config file source. |
| `build()` | Method on `ConfigBuilder` | Builds `Config` from sources, returns `Result<Config, ConfigError>`. |
| `ConfigItem` | Trait | Marker trait for config structs, provides `key()`. |
| `config` | Attribute macro (feature: macro) | Derives `ConfigItem` and sets the TOML key. |

## Features

- Environment variable interpolation: `${VAR}` or `${VAR:default}`.
- Deep merging of config sources.
- Optional validation with `validator` crate.
- Optional macro for easy struct setup.</content>
<parameter name="filePath">thisconfig/README.md