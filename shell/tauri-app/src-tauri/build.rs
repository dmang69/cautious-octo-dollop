fn main() {
    tauri_build::build();
    std::env::set_var("PROTOC", protoc_bin_vendored::protoc_bin_path().unwrap());
    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .compile_protos(&["proto/intentkernel.proto"], &["proto"])
        .expect("failed to compile protos");
}