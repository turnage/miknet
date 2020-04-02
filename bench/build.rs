use std::fs::File;

use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let mut build_log =
        File::create("build_log.txt").expect("opening build log");

    println!("cargo:rerun-if-env-changed=ENET");
    let mut run_in_enet = |cmd: &mut Command| {
        let output = cmd
            .current_dir("third_party/enet")
            .output()
            .expect("command");
        write!(
            build_log,
            "Command output: {}\nCommand error: {}",
            String::from_utf8(output.stdout.clone()).expect("utf8"),
            String::from_utf8(output.stderr.clone()).expect("utf8")
        )
        .expect("writing to build log");
        output
    };
    run_in_enet(Command::new("autoreconf").arg("-vfi"));

    let mut path = PathBuf::new();
    path.push(
        String::from_utf8(run_in_enet(&mut Command::new("pwd")).stdout)
            .expect("valid utf8 path")
            .trim(),
    );
    path.push("build");

    let target_dir_arg = format!("--prefix={:?}", path);
    run_in_enet(Command::new("./configure").arg(target_dir_arg));
    run_in_enet(&mut Command::new("make"));
    run_in_enet(Command::new("make").arg("install"));

    let lib_path = {
        let mut lib_path = path.clone();
        lib_path.push("lib");
        lib_path
    };
    println!(
        "cargo:rustc-link-search=native={}",
        lib_path.as_path().display()
    );
    println!("cargo:rustc-link-lib=static=enet");

    let include_path = {
        let mut include_path = path.clone();
        include_path.push("include");
        include_path
    };
    let enet_header_path = {
        let mut enet_header_path = include_path.clone();
        enet_header_path.push("enet");
        enet_header_path.push("enet.h");
        enet_header_path
    };
    let bindings = bindgen::Builder::default()
        .clang_arg(format!("-I{}", include_path.as_path().display()))
        .header(format!("{}", enet_header_path.as_path().display()))
        .generate()
        .expect("Enet bindgen output");

    let out_dir = std::env::var("OUT_DIR").expect("build out dir");
    bindings
        .write_to_file(PathBuf::from(out_dir).join("enet.rs"))
        .expect("writing bindings");
}
