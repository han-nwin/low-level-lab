// A trait defines the functionality a particular type has and can share with other types.
// Similar to interface in other languages, but more powerful
pub trait Summary {
    fn summarize_author(&self) -> String;

    fn summarize(&self) -> String {
        format!("(Read more from {}...)", self.summarize_author())
    }
}

pub struct NewsArticle {
    pub headline: String,
    pub location: String,
    pub author: String,
    pub content: String,
}

impl Summary for NewsArticle {
    fn summarize_author(&self) -> String {
        format!("@{}", self.author)
    }
    // fn summarize(&self) -> String {
    //     format!("{}, by {} ({})", self.headline, self.author, self.location)
    // }
}

pub struct SocialPost {
    pub username: String,
    pub content: String,
    pub reply: bool,
    pub repost: bool,
}

impl Summary for SocialPost {
    fn summarize_author(&self) -> String {
        format!("@{}", self.username)
    }

    fn summarize(&self) -> String {
        format!("{}: {}", self.username, self.content)
    }
}

// USE trait as parameter
// implement a function for any type that has this trait
pub fn notify(item: &impl Summary) {
    println!("Breaking news! {}", item.summarize());
}

// TRAIT BOUND

// The one above is just syntax sugar for
pub fn notify<T: Summary>(item: &T) {
    println!("Breaking news! {}", item.summarize());
}

//BUT
pub fn notify(item1: &impl Summary, item2: &impl Summary) {}
// -> allow item1 and item2 to have different types as long as they implement Summary trait

pub fn notify<T: Summary>(item1: &T, item2: &T) {}
// force item1 and and item2 to have the same concrete type

//Multiple Trait Bounds with the + Syntax
pub fn notify(item: &(impl Summary + Display)) {}
pub fn notify<T: Summary + Display>(item: &T) {}

//Clearer Trait Bounds with where Clauses
fn some_function<T: Display + Clone, U: Clone + Debug>(t: &T, u: &U) -> i32 {} //-> this is UGLY

fn some_function<T, U>(t: &T, u: &U) -> i32
where
    T: Display + Clone,
    U: Clone + Debug,
{
}

//Returning Types That Implement Traits
fn returns_summarizable() -> impl Summary {
    SocialPost {
        username: String::from("horse_ebooks"),
        content: String::from("of course, as you probably already know, people"),
        reply: false,
        repost: false,
    }
} // By using impl Summary for the return type,
// we specify that the returns_summarizable function returns some
// type that implements the Summary trait without naming the concrete type
// aka SocialPost implement Summary trait

// NOTES: we can NOT do this, since the compiler only pickup 1
fn returns_summarizable(switch: bool) -> impl Summary {
    if switch {
        NewsArticle {
            headline: String::from("Penguins win the Stanley Cup Championship!"),
            location: String::from("Pittsburgh, PA, USA"),
            author: String::from("Iceburgh"),
            content: String::from(
                "The Pittsburgh Penguins once again are the best \
                 hockey team in the NHL.",
            ),
        }
    } else {
        SocialPost {
            username: String::from("horse_ebooks"),
            content: String::from("of course, as you probably already know, people"),
            reply: false,
            repost: false,
        }
    }
}
