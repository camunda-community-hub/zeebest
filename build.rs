use std::fs;

fn main() {
    println!("cargo:rerun-if-changed=proto/gateway.proto");
    fs::create_dir_all("target/proto/build").unwrap();
    tonic_build::configure()
        .out_dir("target/proto/build")
        .compile(&["proto/gateway.proto"], &["proto"])
        .unwrap();
}
