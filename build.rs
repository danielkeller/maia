use std::path::Path;
use std::env;

fn main() {
    let dir = env::var("CARGO_MANIFEST_DIR").unwrap();
	#[cfg(target_arch = "x86")]
	{
		println!("cargo:rustc-link-search=native={}", Path::new(&dir).join("lib/x86").display());
	}
	#[cfg(target_arch = "x86_64")]
	{
		println!("cargo:rustc-link-search=native={}", Path::new(&dir).join("lib/x64").display());
	}
}