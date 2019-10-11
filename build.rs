fn main() {
    println!("cargo:rerun-if-changed=proto/gateway.proto");
    tonic_build::compile_protos("proto/gateway.proto").unwrap();
}
