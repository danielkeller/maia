//! # MacOS-specific Instructions
//! Bundling a Vulkan .app for MacOS requires a few extra steps. The .app bundle is really just a folder, with a structure like
//!
//! * Foo.app
//!     * Contents
//!         * Info.plist
//!         * MacOS
//!             * foo
//!         * Frameworks
//!             * libvulkan.1.dylib
//!             * libvulkan.1.2.198.dylib
//!             * libMoltenVK.dylib
//!         * Resources
//!             * vulkan
//!                 * icd.d
//!                     * MoltenVK_icd.json
//!
//! Info.plist contains the metadata for the app. The minimum information required is the executable name and an app identifier:
//! ```xml
//! <?xml version="1.0" encoding="UTF-8"?>
//! <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
//!     "http://www.apple.com/DTDs///! PropertyList-1.0.dtd">
//! <plist version="1.0">
//! <dict>
//! 	<key>CFBundleExecutable</key>
//! 	<string>foo</string>
//! 	<key>CFBundleIdentifier</key>
//! 	<string>com.awesome.foo</string>
//! </dict>
//! </plist>
//! ```
//!
//! Libvulkan asks the linker to search for it with a path like `@rpath/libvulkan.1.dylib`, where `@rpath` is stored in the executable. To set `@rpath` appropriately, use a build.rs like the following:
//! ```rust
//! fn main() {
//!     #[cfg(target_os = "macos")]
//!     {
//!         println!("cargo:rustc-link-arg=-rpath");
//!         println!("cargo:rustc-link-arg=@executable_path/../Frameworks");
//!     }
//! }
//! ```
//! For `libvulkan` (the loader) to find `libMoltenVK` (the implementation), it needs an "icd" file. It will automatically search the current .app bundle for one, and we can use it to point to the MoltenVK dylib inside our bundle (library_path is relative to the icd file), like so:
//!
//! ```json
//! {
//!     "file_format_version": "1.0.0",
//!     "ICD": {
//!         "library_path": "../../../Frameworks/libMoltenVK.dylib",
//!         "api_version": "1.2.0",
//!         "is_portability_driver": true
//!     }
//! }
//! ```
//!
//! The actual path of the Vulkan dylib can be found using the linker with the "trace" option, and the preferred search path found using objdump. Since this file is a symbolic link to the real dylib, that must be included as well. The MoltenVK dylib can be found using the linker in the same way. So a complete shell script to build an .app bundle looks like the following:
//!
//! ```shell
#![doc = include_str!("../demo/bundle")]
//! ```
