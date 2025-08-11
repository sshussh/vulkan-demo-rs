# Vulkan Triangle Demo in Rust

This is a complete Vulkan demo written in Rust that renders a colorful triangle. It demonstrates the fundamental concepts of Vulkan graphics programming including instance creation, device selection, swapchain setup, render passes, graphics pipelines, and command buffer recording.

## Prerequisites

Before running this demo, you need:

1. **Vulkan SDK**: Install the Vulkan SDK from [LunarG](https://vulkan.lunarg.com/)
   - On Arch: `sudo pacman -S vulkan-devel vulkan-tools vulkan-validation-layers spirv-tools`

2. **Graphics Drivers**: Ensure you have Vulkan-compatible graphics drivers installed
   - NVIDIA: Latest GeForce drivers
   - AMD: AMDGPU-PRO or Mesa drivers
   - Intel: Mesa drivers

3. **Rust**: Install Rust from [rustup.rs](https://www.rust-lang.org/tools/install)

4. **GLSL Compiler**: The `glslc` compiler (part of the Vulkan SDK) for compiling shaders

## Running the Demo

```bash
git clone <repository-url>
cd vulkan-demo
make
```

## What You'll See

The demo will open a window displaying a triangle with the following features:

- **Red vertex** at the top
- **Green vertex** at the bottom right
- **Blue vertex** at the bottom left
- **Smooth color interpolation** between vertices
- **Black background**

## Technical Details

### Dependencies

- **ash**: Vulkan bindings for Rust
- **ash-window**: Platform-specific surface creation
- **winit**: Cross-platform window creation and event handling
- **raw-window-handle**: Window handle abstraction

### Vulkan Features Demonstrated

1. **Instance Creation**: Setting up the Vulkan instance with validation layers
2. **Device Selection**: Choosing an appropriate physical device (GPU)
3. **Logical Device**: Creating a logical device and queues
4. **Surface Creation**: Platform-specific window surface creation
5. **Swapchain**: Setting up the presentation swapchain
6. **Render Pass**: Defining how rendering operations are structured
7. **Graphics Pipeline**: Creating a complete graphics pipeline with shaders
8. **Command Buffers**: Recording and submitting GPU commands
9. **Synchronization**: Using semaphores and fences for proper synchronization
10. **Vertex Data**: Hardcoded triangle vertices with colors

### Steps Performed

- Create instance
- Set up validation layers
- Pick physical device
- Create logical device
- Create surface
- Create swapchain
- Create image views
- Create render pass
- Compile shaders
- Create graphics pipeline
- Create framebuffers
- Create command pool
- Create command buffers
- Create semaphores and fences
- Record commands
- Submit and present
- Clean up everything

### Validation Layers

The demo includes Vulkan validation layers in debug builds, which provide:

- API usage validation
- Memory leak detection
- Performance warnings
- Detailed error messages

## Learning Resources

To learn more about Vulkan and graphics programming:

- [Vulkan Tutorial](https://vulkan-tutorial.com/) - Comprehensive Vulkan learning resource
- [Vulkan Specification](https://www.khronos.org/registry/vulkan/) - Official Vulkan documentation
- [Ash Documentation](https://docs.rs/ash/) - Rust Vulkan bindings documentation
- [Learn OpenGL](https://learnopengl.com/) - Graphics programming fundamentals

## License

This demo is licensed under the GNU General Public License v3.0 (GPL-3.0). You are free to use, modify, and distribute this software under the terms of the GPL. See the LICENSE file for full license text.
