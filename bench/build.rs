use std::fs::File;

use duct::cmd;
use std::ffi::OsString;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

fn run_in(dir: &PathBuf, invocation: &str) {
    let context =
        format!("Running `{}` in `{}`", invocation, dir.as_path().display());
    let invocation: Vec<&str> = invocation.split_whitespace().collect();
    let output = cmd(invocation[0], &invocation[1..])
        .dir(dir)
        .run()
        .expect(&context);
    if !output.status.success() {
        let stderr =
            String::from_utf8(output.stderr).expect("Parsing stderr as utf8");
        panic!("`{}` executed, but failed: {}", context, stderr);
    }
}

fn build_enet(enet_dir: PathBuf) {
    println!("cargo:rerun-if-env-changed=ENET");

    run_in(&enet_dir, "autoreconf -vfi");

    let mut build_path = enet_dir.clone();
    build_path.push("build");

    let target_dir_arg = format!("--prefix={:?}", build_path);
    run_in(
        &enet_dir,
        &format!(
            "{}/configure --prefix={}",
            enet_dir.as_path().display(),
            build_path.as_path().display(),
        ),
    );
    run_in(&enet_dir, "make");
    run_in(&enet_dir, "make install");

    let lib_path = {
        let mut lib_path = build_path.clone();
        lib_path.push("lib");
        lib_path
    };
    println!(
        "cargo:rustc-link-search=native={}",
        lib_path.as_path().display()
    );
    println!("cargo:rustc-link-lib=static=enet");

    let include_path = {
        let mut include_path = build_path.clone();
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

fn build_kcp(kcp_dir: PathBuf) {
    println!("cargo:rerun-if-env-changed=KCP");

    run_in(&kcp_dir, "cmake .");
    run_in(&kcp_dir, "make");

    println!(
        "cargo:rustc-link-search=native={}",
        kcp_dir.as_path().display()
    );
    println!("cargo:rustc-link-lib=static=kcp");

    let bindings = bindgen::Builder::default()
        .header(format!("{}/ikcp.h", kcp_dir.as_path().display()))
        .generate()
        .expect("Enet bindgen output");

    let out_dir = std::env::var("OUT_DIR").expect("build out dir");
    bindings
        .write_to_file(PathBuf::from(out_dir).join("kcp.rs"))
        .expect("writing bindings");
}

fn main() {
    build_enet(
        format!("{}/third_party/enet", env!("CARGO_MANIFEST_DIR")).into(),
    );

    build_kcp(format!("{}/third_party/kcp", env!("CARGO_MANIFEST_DIR")).into());
}
