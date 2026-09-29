# Git Conventions

## Commit Rules

- Do not commit code automatically unless explicitly requested
- Ensure the code runs correctly before committing
- If already in a branch commit there, else, commit in the main/master branch

## Commit Message Format

```
<type>[<scope>]: <subject>
```

A space follows the colon. Type values:

| type | Purpose |
|------|---------|
| feat | New feature |
| fix | Bug fix |
| docs | Documentation or comments |
| style | Code formatting (no runtime impact) |
| refactor | Refactoring (not a new feature or bug fix) |
| perf | Performance optimization |
| test | Adding tests |
| chore | Build process or tooling changes |

Additionally, ensure that the first letter of type, scope, and subject are capitalized.

## Squash Commits and Pull Requests

PR Message/Title Format

PR's MUST use squash commit formatting as follows

```
[Scope] <overview of changes for this PR>
```

Note that the first letter of the scope and commit message must be capitalized for squash commits. Normal commits will have the scope and first letter of a short commit message me uncapitalized

For example:

```
[Shooter] Add different shooting angles
```

This allows us to squash multiple commits that were in the Pull Request into one general commit.

## Other

- You MUST also use squash commit formatting if your commit covers more than one changed issue. Refer to the [squash commit](#squash-commits-and-pull-requests)
- Keep commit names short and elaborate. They're meant to be used to quickly identify issues or to understand changes.

## Build & Test Commands
- Build: `cargo build`
- Test: `cargo test`
- Test with output: `cargo test -- --nocapture`
- Lint: `cargo clippy -- -D warnings`
- Format: `cargo fmt`
- Check (fast, no codegen): `cargo check`
- Audit dependencies: `cargo audit`
- Documentation: `cargo doc --open`

## Rust Edition and Toolchain
- Edition: 2021 (always specify in Cargo.toml)
- MSRV: 1.75.0 (stable, pinned in rust-toolchain.toml)
- Do NOT use nightly features

## Error Handling
- Libraries: use `thiserror` — derive `Error` for all custom error types
- Applications/binaries: use `anyhow` — propagate with `?`, add context with `.context()`
- Never use `.unwrap()` or `.expect()` in library code
- `.expect()` is acceptable in binary main() for setup failures (file not found, etc.)
- Never use `.unwrap()` in tests — use `?` with `#[tokio::test]` or return `Result<(), Box<dyn Error>>`
- Propagate errors with `?` unless there is a documented reason to handle locally

## Clippy Policy
- Treat all warnings as errors: `cargo clippy -- -D warnings`
- Run clippy before committing, not just before merging
- Do not use `#[allow(clippy::...)]` without a comment explaining why

## Code Style
- All public items (structs, enums, functions, traits, modules) must have doc comments (`///`)
- Private items: doc comments are encouraged but not required
- Prefer explicit type annotations on public function signatures
- Do not use wildcard imports (`use foo::*`) except in test modules

## Dependency Policy
- Prefer minimal dependencies — justify each new crate in the PR description
- Pin exact versions for production binaries (`=1.2.3` in Cargo.toml)
- Use `^` (caret) for library crates (allows compatible updates)
- Run `cargo audit` before every release
- Do not add a crate to solve a problem that can be solved with std in under 20 lines

## Testing
- Unit tests: `#[cfg(test)]` module at the bottom of each file
- Integration tests: `tests/` directory
- Test naming: `test_<function_name>_<scenario>` pattern
- Use `cargo test -- --nocapture` when debugging output
- Mock external dependencies using trait objects, not concrete types
- Aim for 80%+ coverage on library crates; focus on edge cases not happy paths

## Memory Safety and Unsafe
- No `unsafe` blocks without a `// SAFETY:` comment explaining the invariants upheld
- If `unsafe` is required, isolate it in a dedicated module (e.g., `src/ffi/` or `src/sys/`)
- Prefer `Arc<Mutex<T>>` over raw pointers for shared state across threads
- Do not use `Rc<RefCell<T>>` in async code (not `Send`)
- If you need `unsafe`, ask before writing it — we may have a safe alternative
