# `cargo-fixeq`

Fix `assert_eq!` test errors by editing the source code.

Inspired by [Mercurial's `run-tests.py -i`](https://www.mercurial-scm.org/repo/hg/rev/02e9355c3420).

## Installation

```bash
cargo install cargo-fixeq
```

## Example

Write tests using `assert_eq!` as usual. Put the code to evaluate on the left, leave a dummy value on the right: 

```rust
fn f(n: usize) -> usize {
    if n <= 2 { 1 } else { f(n - 1) + f(n - 2) }
}

#[test]
fn test_f() {
    assert_eq!(f(10), 0);
    assert_eq!(f(20), 0);
}
```

Run `cargo fixeq`:

```bash
cargo fixeq
```

The dummy values are fixed automatically:

```diff
 fn test_f() {
-    assert_eq!(f(10), 0);
-    assert_eq!(f(20), 0);
+    assert_eq!(f(10), 55);
+    assert_eq!(f(20), 6765);
 }
```

In general, `cargo-fixeq` can be helpful for writing initial tests and update tests. See [here](https://github.com/facebookexperimental/eden/blob/213b3f086c349e84871add20ac8b5641397c62bf/eden/scm/lib/renderdag/src/box_drawing.rs#L321-L340) for a more complicated real world example.

## Command-line Parameters

All parameters are passed to `cargo test`. `cargo-fixeq` does not define its own parameters.
