## Summary

Explain the problem and the focused change.

## Affected Feature

- [ ] Customer flow
- [ ] Admin flow
- [ ] Recommendation production pipeline
- [ ] Controlled experiment
- [ ] Persistence/data
- [ ] Frontend/design system
- [ ] Documentation/CI

## Behaviour and Data Safety

- [ ] Static customer Menu remains unfiltered.
- [ ] Simulation/counterfactual data cannot write to `data/orders.csv`.
- [ ] Timeline clear cannot remove historical orders.
- [ ] Customer and admin sessions remain separate.
- [ ] No credentials, tokens, phone numbers, or private runtime files are added.

## Test Evidence

List commands and manual flows actually run.

```text
cargo fmt --check
cargo check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

## Screenshots

For UI changes, attach phone and desktop screenshots without private data.

## Documentation

List updated documentation, or explain why no documentation change is needed.
