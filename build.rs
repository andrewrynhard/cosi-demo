use prost_build::Config;
use std::fs;

fn main() {
    fs::create_dir_all("./src/spec").expect("failed to create spec output directory");

    let mut cfg = Config::new();
    cfg.protoc_arg("--experimental_allow_proto3_optional");

    tonic_build::configure()
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .out_dir("src/spec")
        .compile_with_config(cfg, &["proto/spec.proto"], &["proto"])
        .expect("failed to generate spec");
}
