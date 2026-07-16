fn main() {
    use std::collections::HashMap;

    let mut scores = HashMap::new();

    // Implicitly defines HashMap<String, i32>
    scores.insert(String::from("Blue"), 10);
    scores.insert(String::from("Yellow"), 50);
    println!("{scores:?}");

    let team_name = String::from("Blue");
    //Here, score will have the value that’s associated with the Blue team,
    //and the result will be 10. The get method returns an Option<&V>;
    //if there’s no value for that key in the hash map, get will return None.
    //This program handles the Option by calling copied to get an Option<i32>
    //rather than an Option<&i32>, then unwrap_or to set score to zero if score
    // doesn’t have an entry for the key.
    let score = scores.get(&team_name).copied().unwrap_or(0);
    println!("{score}");

    // There's no Red so it'll return 0
    let score2 = scores.get("Red").copied().unwrap_or(0);
    println!("{:?}", score2);

    // return None here if no unwrap_or
    let score2 = scores.get("Red").copied();
    println!("{:?}", score2);

    // Iterating
    for (key, value) in &scores {
        println!("{key}: {value}");
    }

    // Insert again will update the current value
    scores.insert(String::from("Blue"), 2384);
    println!("{scores:?}");

    // Insert only if key doesn't exist
    scores.entry(String::from("Green")).or_insert(999);
    println!("{scores:?}");

    // Update a value based on old value
    let text = "hello world wonderful world";

    let mut map = HashMap::new();

    for word in text.split_whitespace() {
        // or_insert(0) is the thing that returns &mut i32. A mutable reference
        let count = map.entry(word).or_insert(0);
        // we use it here to access the old value and update it
        *count += 1;
    }
    println!("{map:?}");
}
