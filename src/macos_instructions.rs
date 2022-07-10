// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

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
//! Info.plist contains the metadata for the app. (See [Apple's documentation](https://developer.apple.com/library/archive/documentation/General/Reference/InfoPlistKeyReference/Articles/AboutInformationPropertyListFiles.html) for more info.) The minimum information required is the executable name and an app identifier.
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
//! The runtime search path for libvulkan is relative to the `@rpath` is stored in the executable. To set `@rpath` appropriately, use a [build.rs](https://doc.rust-lang.org/cargo/reference/build-scripts.html) like the following:
//! ```rust
//! fn main() {
//!     #[cfg(target_os = "macos")]
//!     {
//!         println!("cargo:rustc-link-arg=-rpath");
//!         println!("cargo:rustc-link-arg=@executable_path/../Frameworks");
//!     }
//! }
//! ```
//! For `libvulkan` (the [Vulkan loader](https://github.com/KhronosGroup/Vulkan-Loader/blob/master/docs/LoaderInterfaceArchitecture.md)) to find `libMoltenVK` (the implementation), it needs an "icd" file in the app bundle, like so:
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
//! The shell script given below copies these files from the macos/ folder of the crate, as shown in the [demo project](https://github.com/danielkeller/maia/tree/main/demo).
//!
//! The paths to the dynamic libraries can be found with the linker. So a complete shell script to build an .app bundle looks like the following (with the variables at the top modified appropriately for your package):
//!
//! ```shell
//! #!/bin/sh
//!
//! # Change for your app
//! TARGET=../target
//! BINARY=$TARGET/release/demo
//! OUT=$TARGET/release/Demo.app
//!
//! cargo build --release || exit 1
//!
//! which_lib() {
//!     ld -t -dylib -o /dev/null -arch x86_64 -macosx_version_min 10.12.0 -l$1
//! }
//! soname() {
//!     objdump -p $1 | sed -nr 's/^.*name @rpath\/(.*) \(.*$/\1/p'
//! }
//!
//! # Find the vulkan dylib
//! LINKED="$(which_lib vulkan)"
//! LIBDIR="$(dirname $LINKED)"
//! VK_SONAME="$LIBDIR/$(soname $LINKED)"
//! VK_REALNAME="$LIBDIR/$(readlink $VK_SONAME)"
//! MOLTENVK="$(which_lib MoltenVK)"
//!
//! rm -Rf $OUT
//! mkdir -p $OUT/Contents/MacOS
//! mkdir -p $OUT/Contents/Frameworks
//! mkdir -p $OUT/Contents/Resources/vulkan/icd.d
//!
//! cp macos/Info.plist $OUT/Contents/
//! cp macos/MoltenVK_icd.json $OUT/Contents/Resources/vulkan/icd.d
//! cp $BINARY $OUT/Contents/MacOS
//! cp -R assets $OUT/Contents/Resources/
//! cp $VK_SONAME $OUT/Contents/Frameworks
//! cp $VK_REALNAME $OUT/Contents/Frameworks
//! cp $MOLTENVK $OUT/Contents/Frameworks
//! ```
