# Ember

Safe, low-level Vulkan bindings. The general properties of this library are

1. Memory safe on the host. No safe operation can cause memory corruption or data races in host memory.
2. Generally 1-1 correspondance with Vulkan API calls. In particular, calls which don't allocate or lock in Vulkan also don't do those things in Ember.
3. Usage is idiomatic and hard to get wrong.
4. As ergonomic as possible.