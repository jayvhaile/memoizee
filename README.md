# Memoizee

`Memoizee` is a Rust library designed to simplify caching for both synchronous and asynchronous functions. With just a procedural macro, you can avoid redundant computations and improve performance effortlessly.

## Features

- **Synchronous memoization**: Automatically cache results for synchronous functions.
- **Asynchronous memoization**: Cache results for `async` functions with ease.
- **Simple to use**: Add the `#[memoize]` attribute to your functions, and you're done!

## Installation

Add `memoizee` and `once_cell` to your `Cargo.toml`:

```toml
[dependencies]
memoizee = "0.1.0"
once_cell = "1.20.2"
```

## Usage

Here's how you can use `memoizee` to cache the results of your synchronous and asynchronous functions:

```rust
use memoizee::memoize;

#[memoize]
fn compute_sync(x: u32) -> u32 {
    println!("Computing sync...");
    x * x
}

#[memoize]
async fn compute_async(x: u32) -> u32 {
    println!("Computing async...");
    x * x
}

#[tokio::main]
async fn main() {
    // Test async memoization
    assert_eq!(compute_async(3).await, 9); // Prints: "Computing async..."
    assert_eq!(compute_async(3).await, 9); // Cached, no print

    // Test sync memoization
    assert_eq!(compute_sync(3), 9); // Prints: "Computing sync..."
    assert_eq!(compute_sync(3), 9); // Cached, no print
}
```

## How It Works

1. Add the `#[memoize]` attribute to your function.
2. The macro generates a static memoizer that stores previously computed results.
3. For synchronous functions, results are cached using a lightweight in-memory store.
4. For asynchronous functions, `memoizee` handles futures and caches their resolved values.

## Benefits

- **Performance**: Avoid redundant computations for frequently called functions.
- **Ease of use**: Requires no boilerplate—just annotate your functions with `#[memoize]`.
- **Flexibility**: Works seamlessly with both sync and async functions.

## Limitations

- Currently, `#[memoize]` only supports functions with a single argument.
- The argument type must implement `Clone`, `Eq`, and `Hash`.
- Return values must implement `Clone`.

## Contributing

Contributions are welcome! Feel free to open an issue or submit a pull request if you have ideas for improvements or bug fixes.

## License

This project is licensed under the [MIT License](LICENSE).