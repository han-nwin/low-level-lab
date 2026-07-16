struct User {
    active: bool,
    username: String,
    email: String,
    sign_in_count: u64,
}

struct Color(i32, i32, i32);
struct Point(i32, i32, i32);

//unit-like struct
struct AlwaysEqual;

fn main() {
    let mut user1 = User {
        active: true,
        username: String::from("hannwin"),
        email: String::from("hannwin@gmail.com"),
        sign_in_count: 1,
    };

    user1.email = String::from("hannwin.dev@gmail.com");

    // spread syntax (like ts)
    let mut user2 = User {
        email: String::from("kid.ariel@gmail.com"),
        ..user1
    };

    let black = Color(0, 0, 0);
    let origin = Point(0, 0, 0);

    let subject = AlwaysEqual;
}

fn build_user(email: String, username: String) -> User {
    User {
        active: true,
        username, // same name so no need username: username
        email,
        sign_in_count: 1,
    }
}

// we can use reference for the Struct data but it requires lifetime
struct User_2 {
    active: bool,
    username: &str, //expected named lifetime parameter
    email: &str,    //expected named lifetime parameter
    sign_in_count: u64,
}

fn main_2() {
    let user1 = User_2 {
        active: true,
        username: "someusername123",
        email: "someone@example.com",
        sign_in_count: 1,
    };
}
