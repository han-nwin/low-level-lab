use lru_cache::cache::LRUCache;
fn main() {
    let mut cache = LRUCache::new(3);

    cache.put(1, 10);
    cache.put(2, 20);
    cache.put(3, 30);

    println!("After inserting 1, 2, 3");
    println!("get 1 = {:?}", cache.get(1)); // Some(10)
    println!("get 2 = {:?}", cache.get(2)); // Some(20)
    println!("get 3 = {:?}", cache.get(3)); // Some(30)
    println!("get 9 = {:?}", cache.get(9)); // None

    cache.put(3, 99);
    println!("After inserting 3 with new value");
    println!("get 1 = {:?}", cache.get(1)); // Some(10)
    println!("get 2 = {:?}", cache.get(2)); // Some(20)
    println!("get 3 (new value) = {:?}", cache.get(3)); // Some(99)

    // Cache is full. Since 1 is least recently used now, this should evict key 1.
    cache.put(4, 40);

    println!("after putting 4:");
    println!("get 1 = {:?}", cache.get(1)); // None if eviction works
    println!("get 2 = {:?}", cache.get(2)); // Some(20)
    println!("get 3 = {:?}", cache.get(3)); // Some(99)
    println!("get 4 = {:?}", cache.get(4)); // Some(40)

    println!("map contents:");
    cache.display();
}
