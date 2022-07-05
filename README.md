# Maia

[![Crates.io](https://img.shields.io/crates/v/maia)](https://crates.io/crates/maia) [![docs.rs](https://img.shields.io/docsrs/maia)](https://docs.rs/maia/) [![GitHub Workflow Status](https://img.shields.io/github/workflow/status/danielkeller/maia/CI)](https://github.com/danielkeller/maia/actions/workflows/ci.yml)

Safe, low-level [Vulkan](https://en.wikipedia.org/wiki/Vulkan) bindings. The general properties of this library are

1. Memory safe on the CPU. No safe operation can cause memory corruption or data races in host<sup>1</sup> memory.
2. Lock-free. Thread safety is handled at compile time with `&mut` rather than with mutexes, to avoid performance surprises.
3. Low-level. Close to 1-1 correspondance with Vulkan API calls. Calls which don't allocate in Vulkan also don't allocate in Maia.
4. Selective. Maia intentionally omits Vulkan features that are not performant, not useful, or are rarely supported. However, APIs are provided to allow extension in downstream crates.
5. As ergonomic as possible. In particular, nearly everything is [`Send`](https://doc.rust-lang.org/std/marker/trait.Send.html) and [`Sync`](https://doc.rust-lang.org/std/marker/trait.Sync.html).

<sup>1</sup> Maia does not try to protect the _contents_ of your buffers, images, and shader variables. This is because doing so is both impractical (when using [index buffers](https://renderdoc.org/vkspec_chunked/chap21.html#vkCmdDrawIndexed) for example) and not neccesary, since these values don't have invalid bit patterns and in particular don't contain pointers. It also allows the implementation to be significantly more efficient, since doing optimal synchronization automatically is very tricky.

## Setup

Maia dynamically links to the system's Vulkan loader, so one must be installed. Instructions for specific systems follow.

To begin using the API, create an instance object with [`vk::Instance::new`](crate::vk::Instance::new()).

To enable validation layers for debugging, set the environment variable `VK_INSTANCE_LAYERS="VK_LAYER_KHRONOS_validation"` or use the [Configurator](https://vulkan.lunarg.com/doc/view/latest/windows/vkconfig.html) GUI.

#### On Linux

To build, install your distro's Vulkan development libaries (eg for Debian, `sudo apt install libvulkan-dev`). You will also probably want to install the validation layers, either from the distro (eg `sudo apt install vulkan-validationlayers`) or by installing the Vulkan SDK.

To run, a Vulkan-compatible graphics driver should suffice.

#### On MacOS

To build, install the [Vulkan SDK](https://vulkan.lunarg.com/sdk/home), and enable the "System Global Files" option during installation.

To run, you will probably want to include the Vulkan loader and MoltenVK into your .app bundle. Full instructions are available [here](crate::macos_instructions), and an example can be found in the `demo/` directory.

#### On Windows

A Vulkan-compatible graphics driver is sufficient to build and run. You will probably want to install the [Vulkan SDK](https://vulkan.lunarg.com/sdk/home) for validation layers, though.

#### To run the demos

To compile shaders in the demos, either CMake or the [Vulkan SDK](https://vulkan.lunarg.com/sdk/home) must be installed. (CMake appears to be the easier approach on Linux, and the Vulkan SDK on other systems.)

## Using Vulkan

This documentation assumes that you already know how Vulkan works. If you're just getting started, I can recommend the [Vulkan Guide](https://vkguide.dev/docs/chapter_1), the [Vulkan Tutorial](https://vulkan-tutorial.com/), or the older, but more detailed [API without Secrets](https://www.intel.com/content/www/us/en/developer/articles/training/api-without-secrets-introduction-to-vulkan-part-1.html). The code that they walk you through will look very similar to the [`demo`](https://github.com/danielkeller/maia/tree/main/demo) and [`hello-triangle`](https://github.com/danielkeller/maia/tree/main/hello-triangle) examples in this repo, so you can look at those alongside the walkthroughs to see what the corresponding Rust functions are called.
