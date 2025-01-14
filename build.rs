extern crate embed_resource;
#[cfg(any(target_os = "linux", target_os = "android"))]
extern crate cc;

#[cfg(any(target_os = "linux", target_os = "android"))]
use std::env;
#[cfg(any(target_os = "linux", target_os = "android"))]
use std::io::Write;
#[cfg(any(target_os = "linux", target_os = "android"))]
use std::path::Path;
#[cfg(any(target_os = "linux", target_os = "android"))]
use std::fs::{self, File};


/// The last line of this, after running it through a preprocessor, will expand to the value of `BLKGETSIZE`
#[cfg(any(target_os = "linux", target_os = "android"))]
static IOCTL_CHECK_SOURCE: &str = r#"
#include <sys/mount.h>

BLKGETSIZE
"#;

/// Replace `{}` with the `BLKGETSIZE` expression from `IOCTL_CHECK_SOURCE`
#[cfg(any(target_os = "linux", target_os = "android"))]
static IOCTL_INCLUDE_SKELETON: &str = r#"
/// Return `device size / 512` (`long *` arg)
static BLKGETSIZE: {type} = {expr} as {type};
"#;


fn main() {
    embed_resources();
    get_ioctl_data();
}

fn embed_resources() {
    embed_resource::compile("http-manifest.rc");
}

#[cfg(not(any(target_os = "linux", target_os = "android")))]
fn get_ioctl_data() {}

#[cfg(any(target_os = "linux", target_os = "android"))]
fn get_ioctl_data() {
    let ioctl_dir = Path::new(&env::var("OUT_DIR").unwrap()).join("ioctl-data");
    fs::create_dir_all(&ioctl_dir).unwrap();

    let ioctl_source = ioctl_dir.join("ioctl.c");
    File::create(&ioctl_source).unwrap().write_all(IOCTL_CHECK_SOURCE.as_bytes()).unwrap();

    let ioctl_preprocessed = String::from_utf8(cc::Build::new().file(ioctl_source).expand()).unwrap();
    let blkgetsize_expr = ioctl_preprocessed.lines().next_back().unwrap().replace("U", "");

    let ioctl_request_type = match &env::var("CARGO_CFG_TARGET_ENV").expect("CARGO_CFG_TARGET_ENV")[..] {
        "musl" => "libc::c_int",
        _ => "libc::c_ulong",
    };

    let ioctl_include = ioctl_dir.join("ioctl.rs");
    File::create(&ioctl_include)
        .unwrap()
        .write_all(IOCTL_INCLUDE_SKELETON.replace("{type}", ioctl_request_type).replace("{expr}", &blkgetsize_expr).as_bytes())
        .unwrap();
}
