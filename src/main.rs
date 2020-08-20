use lib::FileManipulator;

fn main() {
    static TEST_FILE: &str = "test-file.ts";
    static TEST_CONTENT: &str = "// Hello \n
console.log( 'Hello, world!' );";
    let mut some_file = FileManipulator::new(TEST_FILE.to_string(), TEST_CONTENT)
        .expect("Could not create empty file");
    some_file.save().expect("Could not save file");

    some_file
        .insert_between(
            "// HELLO ALERT",
            "alert( 'Hello, world!' );",
            Some("// Hello"),
        )
        .expect("Could not modify file");
    some_file.save().expect("Could not save file");
}
