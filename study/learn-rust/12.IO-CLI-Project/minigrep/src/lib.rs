pub fn search(query: &str, contents: &str) -> Vec<String> {
    let mut results = vec![];

    // highlight the query text
    // Pattern:
    // "\x1b[31mred text\x1b[0m"
    // Means:
    // \x1b[   start ANSI escape
    // 0   reset
    // 1   bold
    // 3   italic
    // 4   underline
    // 40-47   background color
    // 100-107 bright background color
    // 30-37   foreground color
    // m       apply style
    // text    printed text
    // \x1b[0m reset everything

    //  You can combine codes with ;:
    // "\x1b[1;31mbold red\x1b[0m"
    // "\x1b[4;32munderlined green\x1b[0m"
    // "\x1b[30;43mblack text on yellow background\x1b[0m"

    results = contents
        .lines()
        .filter(|line| line.contains(query))
        .map(|selected_line| selected_line.replace(query, &format!("\x1b[1;31m{query}\x1b[0m")))
        .collect();

    results
}

pub fn search_case_insensitive(query: &str, contents: &str) -> Vec<String> {
    // rewrite in iterator style
    let results = contents
        .lines()
        // this return Option<String>
        // filter_map won't return things that have None values
        .filter_map(|line| {
            if let Some(index) = line.to_lowercase().find(&query.to_lowercase()) {
                let end = index + query.len();
                let text_to_highlight = line[index..end].to_string();

                Some(line.replace(
                    &text_to_highlight,
                    &format!("\x1b[1;31m{text_to_highlight}\x1b[0m"),
                ))
            } else {
                None
            }
        })
        .collect();

    // for line in contents.lines() {
    //     let lower_line = line.to_lowercase();
    //     let lower_query = query.to_lowercase();
    //
    //     // .find return index: Option<usize>
    //     // It might be:
    //     // Some(0)
    //     // Some(5)
    //     // None
    //     if let Some(index) = lower_line.find(&lower_query) {
    //         let end = index + query.len();
    //
    //         let text_to_highlight = line[index..end].to_string();
    //         let highlighted_line = line.replace(
    //             &text_to_highlight,
    //             &format!("\x1b[1;31m{text_to_highlight}\x1b[0m"),
    //         );
    //         results.push(highlighted_line);
    //     }
    // }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    fn highlight(contents: Vec<&str>, texts_to_highlight: Vec<&str>) -> Vec<String> {
        let mut results = vec![];
        for line in contents {
            let mut highlighted_line = line.to_string();

            // into_iter() called here
            // loop over the reference instead because of the outer loop
            // we don't wanna move every iter
            // loop over the slice
            for text_to_highlight in &texts_to_highlight {
                highlighted_line = highlighted_line.replace(
                    text_to_highlight,
                    &format!("\x1b[1;31m{text_to_highlight}\x1b[0m"),
                );
            }

            results.push(highlighted_line);
        }
        results
    }

    #[test]
    fn case_sensitive() {
        let query = "duct";
        let contents = concat!(
            "Rust:\n",
            "safe, fast, productive.\n",
            "Pick three.\n",
            "Duct Tape"
        );
        assert_eq!(
            highlight(vec!["safe, fast, productive."], vec!["duct"]),
            search(query, contents)
        );
    }

    #[test]
    fn case_insensitive() {
        let query = "rust";
        let contents = concat!(
            "Rust:\n",
            "safe, fast, productive.\n",
            "Pick three.\n",
            "Trust me."
        );
        assert_eq!(
            highlight(vec!["Rust:", "Trust me."], vec!["rust", "Rust"]),
            search_case_insensitive(query, contents)
        );
    }
}
