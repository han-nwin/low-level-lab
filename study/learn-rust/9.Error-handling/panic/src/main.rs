use std::fs::{File, OpenOptions};
use std::io::{self, ErrorKind, Read, Seek, SeekFrom, Write};

fn main() {
    let file_result = File::open("hello.txt");

    let _file = match file_result {
        Ok(file) => file,
        Err(error) => panic!("Problem opening the file: {:?}", error),
    };

    // Handling ErrorKind
    let _greeting_file = match File::open("hello.txt") {
        Ok(file) => file,
        Err(error) => match error.kind() {
            ErrorKind::NotFound => match File::create("hello.txt") {
                Ok(fc) => fc,
                Err(e) => panic!("Problem creating the file: {e:?}"),
            },
            _ => {
                panic!("Problem opening the file: {error:?}");
            }
        },
    };

    // a different syntax
    let _file2 = File::open("hello.txt").unwrap_or_else(|error| {
        if error.kind() == ErrorKind::NotFound {
            File::create("hello.txt").unwrap_or_else(|error| {
                panic!("Error creating file {error:?}");
            })
        } else {
            panic!("Error opening file {error:?}");
        }
    });

    // If we wanna call something on the OK return
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open("hello.txt")
        .unwrap_or_else(|error| {
            panic!("Error opening file {error:?}");
        });

    print_file_contents(&file);
    write_to_file(&file, "I just rewrote this file!").unwrap_or_else(|error| {
        panic!("Error wrting file {error:?}");
    });
    print_file_contents(&file);
    write_to_end_of_file(&file, "I just appended to this file!");
    print_file_contents(&file);

    //reset file
    write_to_file(&file, "Hello World!").unwrap_or_else(|error| {
        panic!("Error wrting file {error:?}");
    });
}

// delete content and write new content at beginning of file
// ? here means we will let the error propagate to the caller and let them handle
fn write_to_file(file: &File, text: &str) -> Result<(), io::Error> {
    let mut file = file.try_clone()?;
    file.set_len(0)?;
    file.seek(SeekFrom::Start(0))?;
    file.write_all(text.as_bytes())?;

    Ok(())
}

// use expect only when we have more information the the compiler
// which is not the case here but still use expect to demonstrate the syntax
fn write_to_end_of_file(file: &File, text: &str) {
    let mut file = file.try_clone().expect("Failed to clone the file handle");
    file.seek(SeekFrom::End(0))
        .expect("Failed to seek to the end of the file");
    file.write_all(text.as_bytes())
        .expect("Failed to write to the file");
}

fn print_file_contents(file: &File) {
    let mut file = file.try_clone().expect("Failed to clone the file handle");
    let mut contents = String::new();
    file.seek(SeekFrom::Start(0))
        .expect("Failed to seek to the beginning of the file");
    file.read_to_string(&mut contents)
        .expect("Failed to read the file");
    println!("{}", contents);
}

// Propagate error
fn read_username_from_file() -> Result<String, io::Error> {
    let username_file_result = File::open("hello.txt");

    // let mut username_file = match username_file_result {
    //     Ok(file) => file,
    //     Err(e) => return Err(e),
    // };

    // or just use ?

    let mut username_file = username_file_result?;

    let mut username = String::new();

    match username_file.read_to_string(&mut username) {
        Ok(_) => Ok(username),
        Err(e) => Err(e),
    }
}
