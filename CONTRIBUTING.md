# Contributing to Axiom Protocol Core

Thank you for your interest in contributing. This document explains the development workflow.

## Development setup

```bash
git clone https://github.com/aethex-foundation/axiom-protocol-core
cd axiom-protocol-core
cargo build
cargo test --all
```

Requirements: **Rust 1.75+**, `rustfmt`, `clippy`.

## Branch workflow

- `main` — stable, always green CI
- `feat/<name>` — feature branches; open a PR against `main`
- `fix/<name>` — bug fix branches

Keep PRs focused on a single change. Rebase on `main` before requesting review.

## Code style

- `cargo fmt --all` before every commit
- Zero `cargo clippy -- -D warnings` violations
- No `unsafe` without a documented safety comment
- No `unwrap()` / `expect()` in library code — return a typed error instead
- Unit tests alongside the code (`#[cfg(test)]`); integration tests under `tests/`

## Commit messages

```
<type>(<scope>): <short summary>

[optional body]
```

Types: `feat`, `fix`, `refactor`, `test`, `docs`, `ci`, `chore`.

Example:
```
feat(consensus): add equivocation detection in VoteAccumulator
```

## Testing

Every public API change requires tests. Run the full suite before pushing:

```bash
cargo test --all --lib
cargo test --test '*'
cargo clippy --all-targets --all-features -- -D warnings
```

## Security issues

Please **do not** open public issues for security vulnerabilities.  
Email **security@aethex.foundation** with a description and reproduction steps.

## License

By contributing you agree that your contributions will be licensed under Apache 2.0.
