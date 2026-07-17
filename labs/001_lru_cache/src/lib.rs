pub mod cache;
pub use cache::LRUCache; //import here and export to outside at the same time

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_none_for_missing_key() {
        let cache = LRUCache::new(2);

        assert_eq!(cache.get(99), None);
    }

    #[test]
    fn inserts_and_reads_values() {
        let mut cache = LRUCache::new(3);

        cache.put(1, 10);
        cache.put(2, 20);
        cache.put(3, 30);

        assert_eq!(cache.get(1), Some(10));
        assert_eq!(cache.get(2), Some(20));
        assert_eq!(cache.get(3), Some(30));
    }

    #[test]
    fn evicts_least_recently_used_key_when_full() {
        let mut cache = LRUCache::new(2);

        cache.put(1, 10);
        cache.put(2, 20);
        cache.put(3, 30);

        assert_eq!(cache.get(1), None);
        assert_eq!(cache.get(2), Some(20));
        assert_eq!(cache.get(3), Some(30));
    }

    #[test]
    fn get_makes_key_recent() {
        let mut cache = LRUCache::new(2);

        cache.put(1, 10);
        cache.put(2, 20);
        assert_eq!(cache.get(1), Some(10));
        cache.put(3, 30);

        assert_eq!(cache.get(1), Some(10));
        assert_eq!(cache.get(2), None);
        assert_eq!(cache.get(3), Some(30));
    }

    #[test]
    fn updating_existing_key_changes_value_without_eviction() {
        let mut cache = LRUCache::new(2);

        cache.put(1, 10);
        cache.put(2, 20);
        cache.put(1, 99);

        assert_eq!(cache.get(1), Some(99));
        assert_eq!(cache.get(2), Some(20));
    }

    #[test]
    fn updating_existing_key_makes_it_recent() {
        let mut cache = LRUCache::new(2);

        cache.put(1, 10);
        cache.put(2, 20);
        cache.put(1, 99);
        cache.put(3, 30);

        assert_eq!(cache.get(1), Some(99));
        assert_eq!(cache.get(2), None);
        assert_eq!(cache.get(3), Some(30));
    }

    #[test]
    fn capacity_one_keeps_only_latest_key() {
        let mut cache = LRUCache::new(1);

        cache.put(1, 10);
        assert_eq!(cache.get(1), Some(10));

        cache.put(2, 20);
        assert_eq!(cache.get(1), None);
        assert_eq!(cache.get(2), Some(20));

        cache.put(2, 99);
        assert_eq!(cache.get(2), Some(99));
    }

    #[test]
    fn repeated_gets_do_not_break_order() {
        let mut cache = LRUCache::new(3);

        cache.put(1, 10);
        cache.put(2, 20);
        cache.put(3, 30);
        assert_eq!(cache.get(1), Some(10));
        assert_eq!(cache.get(1), Some(10));
        assert_eq!(cache.get(2), Some(20));
        cache.put(4, 40);

        assert_eq!(cache.get(1), Some(10));
        assert_eq!(cache.get(2), Some(20));
        assert_eq!(cache.get(3), None);
        assert_eq!(cache.get(4), Some(40));
    }

    #[test]
    #[should_panic(expected = "capacity has to be greater than 0")]
    fn zero_capacity_panics() {
        LRUCache::new(0);
    }
}
