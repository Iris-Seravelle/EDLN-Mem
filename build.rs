fn main() {
    cc::Build::new()
        .file("c_src/eidolon.c")
        .include("c_src")
        .compile("eidolon");

    println!("cargo:rerun-if-changed=c_src/eidolon.c");
    println!("cargo:rerun-if-changed=c_src/eidolon.h");
}
