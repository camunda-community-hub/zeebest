use std::fs;

fn main() {
    //    println!("cargo:rerun-if-changed=proto/gateway.proto");
    fs::create_dir_all("src/proto").unwrap();
    tonic_build::configure()
        .out_dir("src/proto")
        .compile(&["proto/gateway.proto"], &["proto"])
        .unwrap();
}
