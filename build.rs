fn main() {
    cc::Build::new()
        .file("src/sqlite/sqlite.c")
        .compile("sqlite");
}
