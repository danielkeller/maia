# Ember

Safe, low-level Vulkan bindings. The general properties of this library are

1. Memory safe on the host. No safe operation can cause memory corruption or data races in host<sup>1</sup> memory.
2. Wait-free. Synchronization is handled with `&mut` rather than mutexes, to avoid performance surprises. Calls which don't allocate in Vulkan also don't allocate in Ember.
3. Close to 1-1 correspondance with Vulkan API calls.
4. As ergonomic as possible.

<sup>1</sup> Ember does not try to protect the _contents_ of your buffers, images, and shader variables. This is because doing so is both impractical (especially when shaders are involved; for example generating an index buffer or virtual texture in a compute shader would require very complex bounds-checking) and not neccesary, since these values don't have invalid bit patterns and in particular don't contain pointers. However, the exact checks performed are not set in stone; there is the possiblity of adding more checks in the future if needed.
