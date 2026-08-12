# Rust conventions

- Never take two `&mut` into the same collection simultaneously.
  For disjoint element access use `split_at_mut`, `iter_mut`,
  `get_disjoint_mut`, or index-and-copy.
- Prefer restructuring to satisfy the borrow checker over
  reaching for `Rc<RefCell<_>>`.
- Do not use `unsafe` unless explicitly asked.
- Assume the code must pass `cargo clippy -- -D warnings`.
