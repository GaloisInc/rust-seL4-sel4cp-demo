use std::process::Command;
use std::env;
fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    Command::new("make")
        .args(["clean"])
        .output()
        .expect("failed to execute process");
    Command::new("make")
        .args(["libgem.a"])
        .output()
        .expect("failed to execute process");
    Command::new("cp")
        .args(["libgem.a", &(out_dir + "/.") ])
        .output()
        .expect("failed to execute process");
    Command::new("cp")
        .args(["libgem.a", "/work/build/target/release/deps/." ])
        .output()
        .expect("failed to execute process");
}