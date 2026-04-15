# src/logging/

Structured logging built on the `tracing` ecosystem.

## Files

### mod.rs
Exposes `init()` and `init_with_level()`. Wires up a `tracing_subscriber` with a `fmt` layer and an `EnvFilter` that honors the `RUST_LOG` environment variable, defaulting to `info` when unset.

### formatters.rs
Placeholder for custom log formatters. Reserved for future use when the default `fmt` layer stops meeting output requirements (for example, JSON logs or colorized summaries).
