use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque};

fn main() {
    arrays_and_tuples();
    vectors();
    strings();
    queues();
    maps();
    sets();
    priority_queues();
    linked_lists();
}

fn arrays_and_tuples() {
    println!("\n--- Arrays and tuples ---");

    // Array: fixed length and one element type. Its data is stored inline.
    // Use it when the number of values is known at compile time.
    let rgb: [u8; 3] = [255, 128, 0];
    println!("red = {}, full color = {rgb:?}", rgb[0]);

    // Tuple: fixed length, but its fields may have different types.
    // Use it for a small group of related values without named fields.
    // Prefer a struct when field names would make the meaning clearer.
    let user = (42_u32, "Ada", true);
    let (id, name, active) = user;
    println!("id={id}, name={name}, active={active}");

    // Slice: a borrowed view into an array, Vec, or another contiguous sequence.
    // Use &[T] in function parameters when the function only needs to read items.
    let middle: &[u8] = &rgb[1..];
    println!("slice = {middle:?}");
}

fn vectors() {
    println!("\n--- Vec: growable sequence / stack ---");

    // Vec<T> is usually the default collection: ordered, growable, contiguous,
    // fast to index, and efficient when adding/removing at the end.
    let mut scores = vec![10, 20, 30];
    scores.push(40);
    let last = scores.pop();

    // get() returns Option instead of panicking when the index is out of bounds.
    if let Some(second) = scores.get(1) {
        println!("second={second}, popped={last:?}");
    }

    for score in &mut scores {
        *score += 1;
    }
    println!("updated scores = {scores:?}");
}

fn strings() {
    println!("\n--- String and &str ---");

    // String owns growable UTF-8 text; &str borrows a UTF-8 string slice.
    // Rust strings cannot be indexed by integer because UTF-8 characters can
    // occupy different numbers of bytes.
    let mut message = String::from("hello");
    message.push_str(", Rust 🦀");
    print_text(&message); // &String coerces to &str.

    let characters: Vec<char> = message.chars().collect();
    println!("first character = {:?}", characters.first());
}

fn print_text(text: &str) {
    println!("{text}");
}

fn queues() {
    println!("\n--- VecDeque: queue / double-ended queue ---");

    // Use VecDeque when adding or removing at both the front and the back.
    // A Vec is a better default when operations happen only at the back.
    let mut jobs = VecDeque::from(["compile", "test"]);
    jobs.push_back("deploy");
    jobs.push_front("format");

    while let Some(job) = jobs.pop_front() {
        println!("running {job}");
    }
}

fn maps() {
    println!("\n--- HashMap and BTreeMap: key-value lookup ---");

    // HashMap<K, V>: use for fast key lookup when key order does not matter.
    let mut word_counts = HashMap::new();
    for word in "rust makes systems programming accessible rust".split_whitespace() {
        *word_counts.entry(word).or_insert(0) += 1;
    }
    println!("rust count = {:?}", word_counts.get("rust"));

    // Iteration order for HashMap is not guaranteed.
    for (word, count) in &word_counts {
        println!("{word}: {count}");
    }

    // BTreeMap<K, V>: use when keys must stay sorted or for range queries.
    let prices = BTreeMap::from([("apple", 2), ("banana", 1), ("pear", 3)]);
    println!("prices in key order:");
    for (fruit, price) in &prices {
        println!("  {fruit}: ${price}");
    }
    println!(
        "from banana onward = {:?}",
        prices.range("banana"..).collect::<Vec<_>>()
    );
}

fn sets() {
    println!("\n--- HashSet and BTreeSet: unique values ---");

    // HashSet<T>: use for uniqueness and fast membership checks when order does
    // not matter. insert() returns false if the value was already present.
    let mut seen = HashSet::new();
    for id in [3, 1, 3, 2, 1] {
        seen.insert(id);
    }
    println!("unique ids = {seen:?}, contains 2 = {}", seen.contains(&2));

    let admins = HashSet::from([1, 2, 4]);
    let online = HashSet::from([2, 3, 4]);
    println!(
        "admins online = {:?}",
        admins.intersection(&online).collect::<Vec<_>>()
    );

    // BTreeSet<T>: use when unique values also need sorted iteration or ranges.
    let sorted = BTreeSet::from(["pear", "apple", "banana", "apple"]);
    println!("sorted unique fruit = {sorted:?}");
}

fn priority_queues() {
    println!("\n--- BinaryHeap: priority queue ---");

    // BinaryHeap<T> efficiently returns the greatest value first (a max-heap).
    // Use std::cmp::Reverse<T> when a min-heap is needed.
    let mut priorities = BinaryHeap::from([2, 10, 5]);
    while let Some(priority) = priorities.pop() {
        println!("next priority = {priority}");
    }

    let mut smallest_first = BinaryHeap::from([
        std::cmp::Reverse(2),
        std::cmp::Reverse(10),
        std::cmp::Reverse(5),
    ]);
    println!("smallest = {:?}", smallest_first.pop().map(|item| item.0));
}

fn linked_lists() {
    println!("\n--- LinkedList: rarely the best default ---");

    // LinkedList supports insertion/removal at either end, but has no indexing
    // and usually has worse cache performance than Vec or VecDeque. Prefer those
    // unless a linked list's specific operations are required.
    let mut values = LinkedList::new();
    values.push_back(20);
    values.push_front(10);
    println!("linked values = {values:?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry_api_counts_repeated_values() {
        let mut counts = HashMap::new();
        for item in ["a", "b", "a"] {
            *counts.entry(item).or_insert(0) += 1;
        }

        assert_eq!(counts.get("a"), Some(&2));
        assert_eq!(counts.get("b"), Some(&1));
    }

    #[test]
    fn set_operations_find_shared_values() {
        let left = HashSet::from([1, 2, 3]);
        let right = HashSet::from([2, 3, 4]);
        let shared: HashSet<_> = left.intersection(&right).copied().collect();

        assert_eq!(shared, HashSet::from([2, 3]));
    }
}
