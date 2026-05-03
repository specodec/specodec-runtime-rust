use specodec::*;
include!("../gen/test_service_types.rs");
include!("run_emit.rs");
include!("dump_emit.rs");

fn main() {
    let vec_dir = std::path::PathBuf::from(
        std::env::var("VEC_DIR").unwrap_or_else(|_| "/app/vectors".into()));
    let out_dir = std::path::PathBuf::from(
        std::env::var("OUT_DIR").unwrap_or_else(|_| "/app/output_emit_rust".into()));
    std::fs::create_dir_all(&out_dir).unwrap();
    run_emit(&vec_dir, &out_dir);
}
