use dashmap::DashMap;
use std::{future::Future, pin::Pin, sync::Arc};
pub use memoize_macro::memoize;

#[derive(Clone)]
pub struct AsyncMemoizer<K, V>
where
    K: std::hash::Hash + Eq + Send + Sync + Clone + 'static,
    V: Clone + Send + Sync + 'static,
{
    cache: DashMap<K, V>,
    compute_fn: Arc<dyn Fn(K) -> Pin<Box<dyn Future<Output = V> + Send>> + Send + Sync>,
}

impl<K, V> AsyncMemoizer<K, V>
where
    K: std::hash::Hash + Eq + Send + Sync + Clone + 'static,
    V: Clone + Send + Sync + 'static,
{
    pub fn new<F>(compute_fn: F) -> Self
    where
        F: Fn(K) -> Pin<Box<dyn Future<Output = V> + Send>> + Send + Sync + 'static,
    {
        Self {
            cache: DashMap::new(),
            compute_fn: Arc::new(compute_fn),
        }
    }

    pub async fn of(&self, key: K) -> V {
        if let Some(val) = self.cache.get(&key) {
            return val.clone();
        }

        let val = (self.compute_fn)(key.clone()).await;

        self.cache.insert(key, val.clone());
        val
    }
}

#[cfg(test)]
mod async_tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;
    use tokio;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_caching_behavior() {
        let compute_count = Arc::new(AtomicUsize::new(0));

        let compute_fn = {
            let compute_count = Arc::clone(&compute_count);
            move |key: u32| {
                let compute_count = Arc::clone(&compute_count);
                Box::pin(async move {
                    compute_count.fetch_add(1, Ordering::SeqCst);
                    sleep(Duration::from_millis(100)).await;
                    key * key
                }) as Pin<Box<dyn Future<Output = u32> + Send>>
            }
        };

        let memoizer = AsyncMemoizer::new(compute_fn);

        let result1 = memoizer.of(4).await;
        assert_eq!(result1, 16);
        assert_eq!(compute_count.load(Ordering::SeqCst), 1);

        let result2 = memoizer.of(4).await;
        assert_eq!(result2, 16);
        assert_eq!(compute_count.load(Ordering::SeqCst), 1);

        let result3 = memoizer.of(5).await;
        assert_eq!(result3, 25);
        assert_eq!(compute_count.load(Ordering::SeqCst), 2);

        let result4 = memoizer.of(4).await;
        assert_eq!(result4, 16);
        assert_eq!(compute_count.load(Ordering::SeqCst), 2);
    }
}



#[derive(Clone)]
pub struct SyncMemoizer<K, V>
where
    K: std::hash::Hash + Eq + Send + Sync + Clone + 'static,
    V: Clone + Send + Sync + 'static,
{
    cache: DashMap<K, V>,
    compute_fn: Arc<dyn Fn(K) -> V + Send + Sync>,
}

impl<K, V> SyncMemoizer<K, V>
where
    K: std::hash::Hash + Eq + Send + Sync + Clone + 'static,
    V: Clone + Send + Sync + 'static,
{
    pub fn new<F>(compute_fn: F) -> Self
    where
        F: (Fn(K) -> V) + Send + Sync + 'static,
    {
        Self {
            cache: DashMap::new(),
            compute_fn: Arc::new(compute_fn),
        }
    }

    pub fn of(&self, key: K) -> V {
        self.cache.entry(key.clone()).or_insert_with(|| { (self.compute_fn)(key.clone())})
            .value()
            .clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};


    #[test]
    fn test_sync_memoizer_basic() {
        // Counter to track the number of computations
        let computation_count = Arc::new(AtomicUsize::new(0));

        // Create a SyncMemoizer with a computation function
        let compute_fn = {
            let computation_count = computation_count.clone();
            move |key: i32| {
                computation_count.fetch_add(1, Ordering::SeqCst);
                key * 2
            }
        };

        let memoizer = SyncMemoizer::new(compute_fn);

        // Test basic memoization
        assert_eq!(memoizer.of(5), 10); // Computes and caches the result
        assert_eq!(memoizer.of(5), 10); // Should retrieve from the cache
        assert_eq!(computation_count.load(Ordering::SeqCst), 1); // Only one computation

        assert_eq!(memoizer.of(10), 20); // Computes and caches another value
        assert_eq!(memoizer.of(10), 20); // Should retrieve from the cache
        assert_eq!(computation_count.load(Ordering::SeqCst), 2); // Only two computations
    }
}
