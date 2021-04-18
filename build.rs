use flatc_rust;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=fbs/shuttermsg_head.fbs");
    println!("cargo:rerun-if-changed=fbs/shuttermsg.fbs");
    flatc_rust::run(flatc_rust::Args {
        inputs: &[Path::new("fbs/shuttermsg_head.fbs"), Path::new("fbs/shuttermsg.fbs")],
        out_dir: Path::new("target/flatbuffers/"),
        ..Default::default()
    }).expect("flatc invocation");
}
