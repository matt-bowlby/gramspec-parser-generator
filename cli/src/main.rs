use parser::parse_file;

fn main() {
    let file_path = "test_files/gramspec.grm";
    match parse_file(file_path) {
        Ok(gramspec) => {
            println!("Parsed GramSpec successfully!");
            // You can add more code here to work with the parsed GramSpec
        }
        Err(err) => {
            eprintln!("Error parsing GramSpec file: {}", err);
        }
    }
}
