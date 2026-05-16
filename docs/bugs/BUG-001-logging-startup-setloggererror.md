# BUG-001: Logging startup fails with `SetLoggerError(())`

## Summary
Starting the API process could fail before the server booted because the logging bootstrap attempted to register a global logger after one had already been registered.

## Impact
The binary could exit during startup with:

```text
Error: Custom { kind: Other, error: Custom { kind: Other, error: SetLoggerError(()) } }
```

This blocked local development and made observability setup brittle in environments where another logger or subscriber had already been initialized.

## Symptoms
- `cargo run --bin rust-api-starter` exited before the HTTP server started
- no API endpoints became available
- the failure pointed at `SetLoggerError(())` during logging initialization

## Root Cause
`src/logging/mod.rs` called `LogTracer::init()` unconditionally. That function registers a process-wide logger and returns `SetLoggerError` if a logger was already set earlier in the process. The bootstrap treated logging initialization as a hard failure path instead of a best-effort setup.

## Affected Areas
- `src/logging/mod.rs`
- application startup path
- file and console tracing initialization

## Fix Summary
The logging bootstrap was changed to:
- treat `LogTracer::init()` as best effort
- use `tracing_subscriber ... try_init()` as best effort
- keep returning `Ok(())` after the log directory is created

This prevents duplicate global-logger registration from aborting process startup while still enabling logging when the subscriber can be attached.

## Prevention
- Prefer `try_init()` or ignored duplicate-init errors for global process logging setup
- Treat observability bootstrap as non-fatal unless the application explicitly requires startup to fail without logging
- Add startup smoke checks for `cargo run --bin rust-api-starter` after logging changes
- Record logger/subscriber singleton assumptions in code comments or KB notes when touching bootstrap code

## References
- [src/logging/mod.rs](/Users/un/Documents/workshop/rust-api-starter/src/logging/mod.rs)
- [docs/README.md](/Users/un/Documents/workshop/rust-api-starter/docs/README.md)
- [README.md](/Users/un/Documents/workshop/rust-api-starter/README.md)
