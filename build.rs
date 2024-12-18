use std::{
  env, fs,
  path::{Path, PathBuf},
};

fn main() {
  println!("cargo:rustc-link-search=resources");

  let src_dir = Path::new("resources");
  let out_dir: PathBuf =
    PathBuf::from(env::var("OUT_DIR").unwrap_or_else(|_| String::from("target/debug")));

  fs::create_dir_all(out_dir.clone()).expect("Failed to create output directory");

  fs::copy(src_dir.join("sqlite3.dll"), out_dir.join("sqlite3.dll"))
    .expect("Failed to copy sqlite3.dll");
  fs::copy(src_dir.join("sqlite3.lib"), out_dir.join("sqlite3.lib"))
    .expect("Failed to copy sqlite3.lib");
}
