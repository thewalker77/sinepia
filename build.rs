extern crate lalrpop;

fn main() {
    let parser = "src/parser.lalrpop";
    println!("cargo:rerun-if-changed={}", parser);
    lalrpop::Configuration::new()
        .use_cargo_dir_conventions()
        .process_file(parser)
        .unwrap();
}
