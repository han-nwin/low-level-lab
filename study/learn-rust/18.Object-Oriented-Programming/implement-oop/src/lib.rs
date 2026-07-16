pub mod blog {

    pub struct Post {
        state: Option<Box<dyn State>>,
        content: String,
    }

    impl Post {
        pub fn new() -> Self {
            // can use -> Post here too. Self = Post
            Self {
                state: Some(Box::new(Draft {})), // create new Draft
                content: String::new(),
            }
        }

        pub fn add_text(&mut self, text: &str) {
            self.content.push_str(text);
        }

        pub fn content(&self) -> &str {
            // Delegate content rules to the current State, passing this Post for access
            // to its text. `as_ref` borrows the boxed state instead of moving it out of
            // `&self`. `unwrap` is safe because Post methods always leave a state set,
            // and deref coercion dispatches `content` to the State implementation.
            self.state.as_ref().unwrap().content(self)
        }

        pub fn request_review(&mut self) {
            // move the Box state inside Option out of it
            if let Some(s) = self.state.take() {
                self.state = Some(s.request_reviews())
                // change Box state to current state
                // after calling request review
            }
        }

        pub fn approve(&mut self) {
            if let Some(s) = self.state.take() {
                self.state = Some(s.approve())
            }
        }

        pub fn reject(&mut self) {
            if let Some(s) = self.state.take() {
                self.state = Some(s.reject())
            }
        }
    }

    impl Default for Post {
        fn default() -> Self {
            Self::new()
        }
    }

    trait State {
        //NOTE: we have self: Box<Self>. This syntax means the method
        // is only valid when called on a Box holding the
        // type. This syntax takes ownership of Box<Self>,
        // invalidating the old state so that the state value of the Post can transform into a new state.
        fn request_reviews(self: Box<Self>) -> Box<dyn State>; // require that by doing this will change the state
        fn approve(self: Box<Self>) -> Box<dyn State>; // require that by doing this will change the state
        fn reject(self: Box<Self>) -> Box<dyn State>;

        fn content<'a>(&self, _post: &'a Post) -> &'a str {
            ""
        } // if not implement return default empty string
    }

    struct Draft {}

    // NOTE: The request_review method on Draft returns a new, boxed instance of
    // a new PendingReview struct, which represents the state when a post is waiting for a review.
    impl State for Draft {
        fn request_reviews(self: Box<Self>) -> Box<dyn State> {
            Box::new(PendingReview { approve_count: 0 })
        }

        fn approve(self: Box<Self>) -> Box<dyn State> {
            Box::new(Published {})
        }

        fn reject(self: Box<Self>) -> Box<dyn State> {
            self
        }
    }

    struct PendingReview {
        approve_count: u8,
    }

    // NOTE:  it returns itself because when we request a review on a post
    // already in the PendingReview state, it should stay in the PendingReview state.
    impl State for PendingReview {
        fn request_reviews(self: Box<Self>) -> Box<dyn State> {
            self
        }

        fn approve(mut self: Box<Self>) -> Box<dyn State> {
            self.approve_count += 1;
            if self.approve_count >= 2 {
                Box::new(Published {})
            } else {
                self
            }
        }

        fn reject(self: Box<Self>) -> Box<dyn State> {
            Box::new(Draft {})
        }
    }

    struct Published {}

    impl State for Published {
        fn request_reviews(self: Box<Self>) -> Box<dyn State> {
            self
        }

        fn approve(self: Box<Self>) -> Box<dyn State> {
            self
        }

        //override content()
        fn content<'a>(&self, post: &'a Post) -> &'a str {
            &post.content
        }

        fn reject(self: Box<Self>) -> Box<dyn State> {
            self // cannot reject published content
        }
    }
}
