fn main() {
    let list = [1, 2, 3];
    let vec_iter = list.iter();

    for val in vec_iter {
        println!("Got {val}");
    }

    let v1: Vec<i32> = vec![1, 2, 3];

    let v2: Vec<_> = v1.iter().map(|x| x + 1).collect();
    // collect consume the iterator and create
    // new vector
}

// Iterator trait
//pub trait Iterator {
//     type Item;
//
//     fn next(&mut self) -> Option<Self::Item>;
