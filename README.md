# Ember

Safe, low-level Vulkan bindings. The general properties of this library are

1. Memory safe on the host. No safe operation can cause memory corruption or data races in host<sup>1</sup> memory.
2. Wait-free. Synchronization is handled with `&mut` rather than mutexes, to avoid performance surprises. Calls which don't allocate in Vulkan also don't allocate in Ember.
3. Close to 1-1 correspondance with Vulkan API calls.
4. As ergonomic as possible. In particular, nearly everything is [Send] and [Sync].

<sup>1</sup> Ember does not try to protect the _contents_ of your buffers, images, and shader variables. This is because doing so is both impractical (especially when shaders are involved; for example generating an index buffer or virtual texture in a compute shader would require very complex bounds-checking) and not neccesary, since these values don't have invalid bit patterns and in particular don't contain pointers. However, the exact checks performed are not set in stone; there is the possiblity of adding more checks in the future if needed.

## Setup

Ember dynamically links to the system's Vulkan loader.

### On Linux

To build, install your distro's Vulkan development libaries (eg for Debian, `sudo apt install libvulkan-dev`).

To run, a Vulkan-compatible graphics driver should suffice.

### On MacOS

To build, install the [Vulkan SDK](https://vulkan.lunarg.com/sdk/home), and enable the "System Global Files" option during installation.

To run, you will probably want to include the Vulkan loader and MoltenVK into your .app bundle. To do so, add the following to your binary's `build.rs` so the binary can find the dylibs:
```rust
println!("cargo:rustc-link-arg=-rpath");
println!("cargo:rustc-link-arg=@executable_path/../Frameworks");
```
and copy the dylibs into `Frameworks`. An example `build.rs` and shell script to do this can be found in the `demo/` directory.

### On Windows

(todo)
