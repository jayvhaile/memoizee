use memoize::memoize;

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
    assert_eq!(compute_async(3).await, 9);
    assert_eq!(compute_async(3).await, 9); // Cached

    // Test sync memoization
    assert_eq!(compute_sync(3), 9);
    assert_eq!(compute_sync(3), 9); // Cached
}
