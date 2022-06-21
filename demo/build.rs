fn main() {
    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-arg=-rpath");
        println!("cargo:rustc-link-arg=@executable_path/../Frameworks");
    }
}
