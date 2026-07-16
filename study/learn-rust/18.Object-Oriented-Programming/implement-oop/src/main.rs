use implement_oop::blog::Post;

fn main() {
    let mut post = Post::new();

    post.add_text("I ate a salad for lunch today");
    assert_eq!("", post.content());

    post.request_review();
    assert_eq!("", post.content());

    post.reject();
    assert_eq!("", post.content());

    post.request_review();
    assert_eq!("", post.content());

    post.approve(); // first approve won't change it
    assert_eq!("", post.content());

    post.approve(); // second approve will
    assert_eq!("I ate a salad for lunch today", post.content());
}
