fn main() {
    let package = "tree-sitter-c";
    let source_directory = format!("{}/src", package);
    let source_file = format!("{}/parser.c", source_directory);

    // Rerun build script if parser source changes.
    println!("cargo:rerun-if-changed={}", source_file);

    // Compile parser C code into a Rust binary.
    cc::Build::new()
        .include(source_directory)
        .file(source_file)
        .compile(&package);
}
