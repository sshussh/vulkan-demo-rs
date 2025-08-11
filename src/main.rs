use ash::vk;
use ash::{Device, Entry, Instance};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::ffi::{CStr, CString, c_char};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

const VALIDATION_ENABLED: bool = cfg!(debug_assertions);
const VALIDATION_LAYER_NAME: &str = "VK_LAYER_KHRONOS_validation";

#[allow(dead_code)]
struct VulkanApp {
    _entry: Entry,
    instance: Instance,
    debug_utils_loader: ash::extensions::ext::DebugUtils,
    debug_messenger: vk::DebugUtilsMessengerEXT,
    surface: vk::SurfaceKHR,
    surface_loader: ash::extensions::khr::Surface,
    physical_device: vk::PhysicalDevice,
    device: Device,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
    swapchain_loader: ash::extensions::khr::Swapchain,
    swapchain: vk::SwapchainKHR,
    swapchain_images: Vec<vk::Image>,
    swapchain_image_format: vk::Format,
    swapchain_extent: vk::Extent2D,
    swapchain_image_views: Vec<vk::ImageView>,
    render_pass: vk::RenderPass,
    pipeline_layout: vk::PipelineLayout,
    graphics_pipeline: vk::Pipeline,
    framebuffers: Vec<vk::Framebuffer>,
    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,
    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: Vec<vk::Fence>,
    current_frame: usize,
}

const MAX_FRAMES_IN_FLIGHT: usize = 2;

unsafe extern "system" fn debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    unsafe {
        let callback_data = *p_callback_data;
        let message_id_number = callback_data.message_id_number;

        let message_id_name = if callback_data.p_message_id_name.is_null() {
            std::borrow::Cow::from("")
        } else {
            CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
        };

        let message = if callback_data.p_message.is_null() {
            std::borrow::Cow::from("")
        } else {
            CStr::from_ptr(callback_data.p_message).to_string_lossy()
        };

        println!(
            "{:?}:\n{:?} [{} ({})] : {}\n",
            message_severity, message_type, message_id_name, message_id_number, message,
        );

        vk::FALSE
    }
}

impl VulkanApp {
    fn new(window: &Window) -> Self {
        unsafe {
            let entry = Entry::load().unwrap();
            let instance = Self::create_instance(&entry, window);
            let (debug_utils_loader, debug_messenger) =
                Self::setup_debug_messenger(&entry, &instance);
            let surface = Self::create_surface(&entry, &instance, window);
            let surface_loader = ash::extensions::khr::Surface::new(&entry, &instance);
            let physical_device = Self::pick_physical_device(&instance, &surface_loader, surface);
            let (device, graphics_queue, present_queue) =
                Self::create_logical_device(&instance, physical_device, &surface_loader, surface);
            let swapchain_loader = ash::extensions::khr::Swapchain::new(&instance, &device);
            let (swapchain, swapchain_images, swapchain_image_format, swapchain_extent) =
                Self::create_swapchain(
                    window,
                    &instance,
                    &device,
                    physical_device,
                    &surface_loader,
                    surface,
                    &swapchain_loader,
                );
            let swapchain_image_views =
                Self::create_image_views(&device, &swapchain_images, swapchain_image_format);
            let render_pass = Self::create_render_pass(&device, swapchain_image_format);
            let (graphics_pipeline, pipeline_layout) =
                Self::create_graphics_pipeline(&device, render_pass, swapchain_extent);
            let framebuffers = Self::create_framebuffers(
                &device,
                &swapchain_image_views,
                render_pass,
                swapchain_extent,
            );
            let command_pool = Self::create_command_pool(
                &instance,
                &device,
                physical_device,
                &surface_loader,
                surface,
            );
            let command_buffers = Self::create_command_buffers(&device, command_pool);
            let (image_available_semaphores, render_finished_semaphores, in_flight_fences) =
                Self::create_sync_objects(&device);

            VulkanApp {
                _entry: entry,
                instance,
                debug_utils_loader,
                debug_messenger,
                surface,
                surface_loader,
                physical_device,
                device,
                graphics_queue,
                present_queue,
                swapchain_loader,
                swapchain,
                swapchain_images,
                swapchain_image_format,
                swapchain_extent,
                swapchain_image_views,
                render_pass,
                pipeline_layout,
                graphics_pipeline,
                framebuffers,
                command_pool,
                command_buffers,
                image_available_semaphores,
                render_finished_semaphores,
                in_flight_fences,
                current_frame: 0,
            }
        }
    }

    unsafe fn create_instance(entry: &Entry, window: &Window) -> Instance {
        let app_name = CString::new("Vulkan Triangle").unwrap();
        let engine_name = CString::new("No Engine").unwrap();
        let app_info = vk::ApplicationInfo::builder()
            .application_name(&app_name)
            .application_version(vk::make_api_version(0, 1, 0, 0))
            .engine_name(&engine_name)
            .engine_version(vk::make_api_version(0, 1, 0, 0))
            .api_version(vk::make_api_version(0, 1, 0, 0));

        let mut extension_names =
            ash_window::enumerate_required_extensions(window.raw_display_handle())
                .unwrap()
                .to_vec();

        if VALIDATION_ENABLED {
            extension_names.push(ash::extensions::ext::DebugUtils::name().as_ptr());
        }

        let layers_names_raw: Vec<CString> = if VALIDATION_ENABLED {
            vec![CString::new(VALIDATION_LAYER_NAME).unwrap()]
        } else {
            vec![]
        };

        let layer_names: Vec<*const c_char> = layers_names_raw
            .iter()
            .map(|raw_name| raw_name.as_ptr())
            .collect();

        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_layer_names(&layer_names)
            .enabled_extension_names(&extension_names);

        unsafe {
            entry
                .create_instance(&create_info, None)
                .expect("Failed to create instance")
        }
    }

    unsafe fn setup_debug_messenger(
        entry: &Entry,
        instance: &Instance,
    ) -> (ash::extensions::ext::DebugUtils, vk::DebugUtilsMessengerEXT) {
        let debug_utils_loader = ash::extensions::ext::DebugUtils::new(entry, instance);

        if !VALIDATION_ENABLED {
            return (debug_utils_loader, vk::DebugUtilsMessengerEXT::null());
        }

        let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            )
            .pfn_user_callback(Some(debug_callback));

        let debug_call_back = unsafe {
            debug_utils_loader
                .create_debug_utils_messenger(&debug_info, None)
                .expect("Failed to set up debug messenger")
        };

        (debug_utils_loader, debug_call_back)
    }

    unsafe fn create_surface(
        entry: &Entry,
        instance: &Instance,
        window: &Window,
    ) -> vk::SurfaceKHR {
        unsafe {
            ash_window::create_surface(
                entry,
                instance,
                window.raw_display_handle(),
                window.raw_window_handle(),
                None,
            )
            .expect("Failed to create window surface")
        }
    }

    unsafe fn pick_physical_device(
        instance: &Instance,
        surface_loader: &ash::extensions::khr::Surface,
        surface: vk::SurfaceKHR,
    ) -> vk::PhysicalDevice {
        let physical_devices = unsafe {
            instance
                .enumerate_physical_devices()
                .expect("Failed to enumerate Physical Devices")
        };

        for &physical_device in physical_devices.iter() {
            if unsafe {
                Self::is_device_suitable(instance, physical_device, surface_loader, surface)
            } {
                return physical_device;
            }
        }

        panic!("Failed to find a suitable GPU");
    }

    unsafe fn is_device_suitable(
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
        surface_loader: &ash::extensions::khr::Surface,
        surface: vk::SurfaceKHR,
    ) -> bool {
        let _device_properties =
            unsafe { instance.get_physical_device_properties(physical_device) };
        let _device_features = unsafe { instance.get_physical_device_features(physical_device) };

        let queue_family_indices = unsafe {
            Self::find_queue_families(instance, physical_device, surface_loader, surface)
        };

        queue_family_indices.is_complete()
    }

    unsafe fn find_queue_families(
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
        surface_loader: &ash::extensions::khr::Surface,
        surface: vk::SurfaceKHR,
    ) -> QueueFamilyIndices {
        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

        let mut queue_family_indices = QueueFamilyIndices::default();

        for (index, queue_family) in queue_families.iter().enumerate() {
            if queue_family.queue_count > 0
                && queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
            {
                queue_family_indices.graphics_family = Some(index as u32);
            }

            let present_support = unsafe {
                surface_loader
                    .get_physical_device_surface_support(physical_device, index as u32, surface)
                    .expect("Failed to check surface support")
            };

            if queue_family.queue_count > 0 && present_support {
                queue_family_indices.present_family = Some(index as u32);
            }

            if queue_family_indices.is_complete() {
                break;
            }
        }

        queue_family_indices
    }

    unsafe fn create_logical_device(
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
        surface_loader: &ash::extensions::khr::Surface,
        surface: vk::SurfaceKHR,
    ) -> (Device, vk::Queue, vk::Queue) {
        let indices = unsafe {
            Self::find_queue_families(instance, physical_device, surface_loader, surface)
        };

        let mut unique_queue_families = std::collections::HashSet::new();
        unique_queue_families.insert(indices.graphics_family.unwrap());
        unique_queue_families.insert(indices.present_family.unwrap());

        let queue_priority = 1.0;
        let queue_create_infos: Vec<vk::DeviceQueueCreateInfo> = unique_queue_families
            .iter()
            .map(|&queue_family| {
                vk::DeviceQueueCreateInfo::builder()
                    .queue_family_index(queue_family)
                    .queue_priorities(&[queue_priority])
                    .build()
            })
            .collect();

        let device_features = vk::PhysicalDeviceFeatures::builder();

        let device_extension_names_raw = [ash::extensions::khr::Swapchain::name()];
        let device_extension_names: Vec<*const c_char> = device_extension_names_raw
            .iter()
            .map(|raw_name| raw_name.as_ptr())
            .collect();

        let device_create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_create_infos)
            .enabled_features(&device_features)
            .enabled_extension_names(&device_extension_names);

        let device = unsafe {
            instance
                .create_device(physical_device, &device_create_info, None)
                .expect("Failed to create logical device")
        };

        let graphics_queue =
            unsafe { device.get_device_queue(indices.graphics_family.unwrap(), 0) };
        let present_queue = unsafe { device.get_device_queue(indices.present_family.unwrap(), 0) };

        (device, graphics_queue, present_queue)
    }

    unsafe fn create_swapchain(
        window: &Window,
        instance: &Instance,
        _device: &Device,
        physical_device: vk::PhysicalDevice,
        surface_loader: &ash::extensions::khr::Surface,
        surface: vk::SurfaceKHR,
        swapchain_loader: &ash::extensions::khr::Swapchain,
    ) -> (vk::SwapchainKHR, Vec<vk::Image>, vk::Format, vk::Extent2D) {
        let swapchain_support =
            unsafe { Self::query_swapchain_support(physical_device, surface_loader, surface) };

        let surface_format = Self::choose_swap_surface_format(&swapchain_support.formats);
        let present_mode = Self::choose_swap_present_mode(&swapchain_support.present_modes);
        let extent = Self::choose_swap_extent(&swapchain_support.capabilities, window);

        let image_count = swapchain_support.capabilities.min_image_count + 1;
        let image_count = if swapchain_support.capabilities.max_image_count > 0 {
            image_count.min(swapchain_support.capabilities.max_image_count)
        } else {
            image_count
        };

        let mut create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface)
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT);

        let indices = unsafe {
            Self::find_queue_families(instance, physical_device, surface_loader, surface)
        };
        let queue_family_indices = [
            indices.graphics_family.unwrap(),
            indices.present_family.unwrap(),
        ];

        if indices.graphics_family != indices.present_family {
            create_info = create_info
                .image_sharing_mode(vk::SharingMode::CONCURRENT)
                .queue_family_indices(&queue_family_indices);
        } else {
            create_info = create_info.image_sharing_mode(vk::SharingMode::EXCLUSIVE);
        }

        create_info = create_info
            .pre_transform(swapchain_support.capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .old_swapchain(vk::SwapchainKHR::null());

        let swapchain = unsafe {
            swapchain_loader
                .create_swapchain(&create_info, None)
                .expect("Failed to create swapchain")
        };

        let swapchain_images = unsafe {
            swapchain_loader
                .get_swapchain_images(swapchain)
                .expect("Failed to get swapchain images")
        };

        (swapchain, swapchain_images, surface_format.format, extent)
    }

    unsafe fn query_swapchain_support(
        physical_device: vk::PhysicalDevice,
        surface_loader: &ash::extensions::khr::Surface,
        surface: vk::SurfaceKHR,
    ) -> SwapchainSupportDetails {
        let capabilities = unsafe {
            surface_loader
                .get_physical_device_surface_capabilities(physical_device, surface)
                .expect("Failed to get surface capabilities")
        };

        let formats = unsafe {
            surface_loader
                .get_physical_device_surface_formats(physical_device, surface)
                .expect("Failed to get surface formats")
        };

        let present_modes = unsafe {
            surface_loader
                .get_physical_device_surface_present_modes(physical_device, surface)
                .expect("Failed to get surface present modes")
        };

        SwapchainSupportDetails {
            capabilities,
            formats,
            present_modes,
        }
    }

    fn choose_swap_surface_format(
        available_formats: &[vk::SurfaceFormatKHR],
    ) -> vk::SurfaceFormatKHR {
        for available_format in available_formats {
            if available_format.format == vk::Format::B8G8R8A8_SRGB
                && available_format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            {
                return *available_format;
            }
        }

        available_formats[0]
    }

    fn choose_swap_present_mode(
        available_present_modes: &[vk::PresentModeKHR],
    ) -> vk::PresentModeKHR {
        for &available_present_mode in available_present_modes.iter() {
            if available_present_mode == vk::PresentModeKHR::MAILBOX {
                return available_present_mode;
            }
        }

        vk::PresentModeKHR::FIFO
    }

    fn choose_swap_extent(
        capabilities: &vk::SurfaceCapabilitiesKHR,
        window: &Window,
    ) -> vk::Extent2D {
        if capabilities.current_extent.width != u32::MAX {
            capabilities.current_extent
        } else {
            let size = window.inner_size();
            let width = size.width.clamp(
                capabilities.min_image_extent.width,
                capabilities.max_image_extent.width,
            );
            let height = size.height.clamp(
                capabilities.min_image_extent.height,
                capabilities.max_image_extent.height,
            );

            vk::Extent2D { width, height }
        }
    }

    unsafe fn create_image_views(
        device: &Device,
        swapchain_images: &[vk::Image],
        swapchain_image_format: vk::Format,
    ) -> Vec<vk::ImageView> {
        swapchain_images
            .iter()
            .map(|&image| {
                let create_info = vk::ImageViewCreateInfo::builder()
                    .image(image)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(swapchain_image_format)
                    .components(vk::ComponentMapping {
                        r: vk::ComponentSwizzle::IDENTITY,
                        g: vk::ComponentSwizzle::IDENTITY,
                        b: vk::ComponentSwizzle::IDENTITY,
                        a: vk::ComponentSwizzle::IDENTITY,
                    })
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    });

                unsafe {
                    device
                        .create_image_view(&create_info, None)
                        .expect("Failed to create image view")
                }
            })
            .collect()
    }

    unsafe fn create_render_pass(
        device: &Device,
        swapchain_image_format: vk::Format,
    ) -> vk::RenderPass {
        let color_attachment = vk::AttachmentDescription::builder()
            .format(swapchain_image_format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

        let color_attachment_ref = vk::AttachmentReference::builder()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let subpass = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(std::slice::from_ref(&color_attachment_ref));

        let dependency = vk::SubpassDependency::builder()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE);

        let render_pass_info = vk::RenderPassCreateInfo::builder()
            .attachments(std::slice::from_ref(&color_attachment))
            .subpasses(std::slice::from_ref(&subpass))
            .dependencies(std::slice::from_ref(&dependency));

        unsafe {
            device
                .create_render_pass(&render_pass_info, None)
                .expect("Failed to create render pass")
        }
    }

    unsafe fn create_graphics_pipeline(
        device: &Device,
        render_pass: vk::RenderPass,
        swapchain_extent: vk::Extent2D,
    ) -> (vk::Pipeline, vk::PipelineLayout) {
        let vert_shader_code = include_bytes!("shaders/vert.spv");
        let frag_shader_code = include_bytes!("shaders/frag.spv");

        let vert_shader_module = unsafe { Self::create_shader_module(device, vert_shader_code) };
        let frag_shader_module = unsafe { Self::create_shader_module(device, frag_shader_code) };

        let main_function_name = CString::new("main").unwrap();

        let vert_shader_stage_info = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vert_shader_module)
            .name(&main_function_name);

        let frag_shader_stage_info = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(frag_shader_module)
            .name(&main_function_name);

        let shader_stages = [
            vert_shader_stage_info.build(),
            frag_shader_stage_info.build(),
        ];

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(&[])
            .vertex_attribute_descriptions(&[]);

        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let viewport = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: swapchain_extent.width as f32,
            height: swapchain_extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };

        let scissor = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: swapchain_extent,
        };

        let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewports(std::slice::from_ref(&viewport))
            .scissors(std::slice::from_ref(&scissor));

        let rasterizer = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false);

        let multisampling = vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .blend_enable(false);

        let color_blending = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(std::slice::from_ref(&color_blend_attachment))
            .blend_constants([0.0, 0.0, 0.0, 0.0]);

        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&[])
            .push_constant_ranges(&[]);

        let pipeline_layout = unsafe {
            device
                .create_pipeline_layout(&pipeline_layout_info, None)
                .expect("Failed to create pipeline layout")
        };

        let pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterizer)
            .multisample_state(&multisampling)
            .color_blend_state(&color_blending)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .subpass(0);

        let graphics_pipeline = unsafe {
            device
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &[pipeline_info.build()],
                    None,
                )
                .expect("Failed to create graphics pipeline")[0]
        };

        unsafe {
            device.destroy_shader_module(frag_shader_module, None);
            device.destroy_shader_module(vert_shader_module, None);
        }

        (graphics_pipeline, pipeline_layout)
    }

    unsafe fn create_shader_module(device: &Device, code: &[u8]) -> vk::ShaderModule {
        let create_info = vk::ShaderModuleCreateInfo::builder().code(unsafe {
            std::slice::from_raw_parts(code.as_ptr() as *const u32, code.len() / 4)
        });

        unsafe {
            device
                .create_shader_module(&create_info, None)
                .expect("Failed to create shader module")
        }
    }

    unsafe fn create_framebuffers(
        device: &Device,
        swapchain_image_views: &[vk::ImageView],
        render_pass: vk::RenderPass,
        swapchain_extent: vk::Extent2D,
    ) -> Vec<vk::Framebuffer> {
        swapchain_image_views
            .iter()
            .map(|&image_view| {
                let attachments = [image_view];
                let framebuffer_info = vk::FramebufferCreateInfo::builder()
                    .render_pass(render_pass)
                    .attachments(&attachments)
                    .width(swapchain_extent.width)
                    .height(swapchain_extent.height)
                    .layers(1);

                unsafe {
                    device
                        .create_framebuffer(&framebuffer_info, None)
                        .expect("Failed to create framebuffer")
                }
            })
            .collect()
    }

    unsafe fn create_command_pool(
        instance: &Instance,
        device: &Device,
        physical_device: vk::PhysicalDevice,
        surface_loader: &ash::extensions::khr::Surface,
        surface: vk::SurfaceKHR,
    ) -> vk::CommandPool {
        let queue_family_indices = unsafe {
            Self::find_queue_families(instance, physical_device, surface_loader, surface)
        };

        let pool_info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(queue_family_indices.graphics_family.unwrap());

        unsafe {
            device
                .create_command_pool(&pool_info, None)
                .expect("Failed to create command pool")
        }
    }

    unsafe fn create_command_buffers(
        device: &Device,
        command_pool: vk::CommandPool,
    ) -> Vec<vk::CommandBuffer> {
        let alloc_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(MAX_FRAMES_IN_FLIGHT as u32);

        unsafe {
            device
                .allocate_command_buffers(&alloc_info)
                .expect("Failed to allocate command buffers")
        }
    }

    unsafe fn create_sync_objects(
        device: &Device,
    ) -> (Vec<vk::Semaphore>, Vec<vk::Semaphore>, Vec<vk::Fence>) {
        let mut image_available_semaphores = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        let mut render_finished_semaphores = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        let mut in_flight_fences = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);

        let semaphore_info = vk::SemaphoreCreateInfo::builder();
        let fence_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);

        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            image_available_semaphores.push(unsafe {
                device
                    .create_semaphore(&semaphore_info, None)
                    .expect("Failed to create semaphore")
            });
            render_finished_semaphores.push(unsafe {
                device
                    .create_semaphore(&semaphore_info, None)
                    .expect("Failed to create semaphore")
            });
            in_flight_fences.push(unsafe {
                device
                    .create_fence(&fence_info, None)
                    .expect("Failed to create fence")
            });
        }

        (
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
        )
    }

    unsafe fn record_command_buffer(&self, command_buffer: vk::CommandBuffer, image_index: usize) {
        let begin_info = vk::CommandBufferBeginInfo::builder();

        unsafe {
            self.device
                .begin_command_buffer(command_buffer, &begin_info)
                .expect("Failed to begin recording command buffer");
        }

        let clear_color = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        };

        let render_pass_info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.render_pass)
            .framebuffer(self.framebuffers[image_index])
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain_extent,
            })
            .clear_values(std::slice::from_ref(&clear_color));

        unsafe {
            self.device.cmd_begin_render_pass(
                command_buffer,
                &render_pass_info,
                vk::SubpassContents::INLINE,
            );

            self.device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.graphics_pipeline,
            );

            self.device.cmd_draw(command_buffer, 3, 1, 0, 0);

            self.device.cmd_end_render_pass(command_buffer);

            self.device
                .end_command_buffer(command_buffer)
                .expect("Failed to record command buffer");
        }
    }

    unsafe fn draw_frame(&mut self) {
        unsafe {
            self.device
                .wait_for_fences(&[self.in_flight_fences[self.current_frame]], true, u64::MAX)
                .expect("Failed to wait for fence");

            self.device
                .reset_fences(&[self.in_flight_fences[self.current_frame]])
                .expect("Failed to reset fence");
        }

        let result = unsafe {
            self.swapchain_loader.acquire_next_image(
                self.swapchain,
                u64::MAX,
                self.image_available_semaphores[self.current_frame],
                vk::Fence::null(),
            )
        };

        let image_index = match result {
            Ok((image_index, _)) => image_index as usize,
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                return;
            }
            Err(error) => panic!("Error while acquiring next image. Cause: {}", error),
        };

        unsafe {
            self.device
                .reset_command_buffer(
                    self.command_buffers[self.current_frame],
                    vk::CommandBufferResetFlags::empty(),
                )
                .expect("Failed to reset command buffer");

            self.record_command_buffer(self.command_buffers[self.current_frame], image_index);
        }

        let wait_semaphores = [self.image_available_semaphores[self.current_frame]];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [self.render_finished_semaphores[self.current_frame]];
        let command_buffers = [self.command_buffers[self.current_frame]];

        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores);

        unsafe {
            self.device
                .queue_submit(
                    self.graphics_queue,
                    &[submit_info.build()],
                    self.in_flight_fences[self.current_frame],
                )
                .expect("Failed to submit draw command buffer");
        }

        let swapchains = [self.swapchain];
        let image_indices = [image_index as u32];

        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&signal_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        let _result = unsafe {
            self.swapchain_loader
                .queue_present(self.present_queue, &present_info)
        };

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
    }
}

impl Drop for VulkanApp {
    fn drop(&mut self) {
        unsafe {
            self.device.device_wait_idle().unwrap();

            for i in 0..MAX_FRAMES_IN_FLIGHT {
                self.device
                    .destroy_semaphore(self.image_available_semaphores[i], None);
                self.device
                    .destroy_semaphore(self.render_finished_semaphores[i], None);
                self.device.destroy_fence(self.in_flight_fences[i], None);
            }

            self.device.destroy_command_pool(self.command_pool, None);

            for &framebuffer in self.framebuffers.iter() {
                self.device.destroy_framebuffer(framebuffer, None);
            }

            self.device.destroy_pipeline(self.graphics_pipeline, None);
            self.device
                .destroy_pipeline_layout(self.pipeline_layout, None);
            self.device.destroy_render_pass(self.render_pass, None);

            for &image_view in self.swapchain_image_views.iter() {
                self.device.destroy_image_view(image_view, None);
            }

            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None);
            self.device.destroy_device(None);

            if VALIDATION_ENABLED {
                self.debug_utils_loader
                    .destroy_debug_utils_messenger(self.debug_messenger, None);
            }

            self.surface_loader.destroy_surface(self.surface, None);
            self.instance.destroy_instance(None);
        }
    }
}

#[derive(Default)]
struct QueueFamilyIndices {
    graphics_family: Option<u32>,
    present_family: Option<u32>,
}

impl QueueFamilyIndices {
    fn is_complete(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }
}

struct SwapchainSupportDetails {
    capabilities: vk::SurfaceCapabilitiesKHR,
    formats: Vec<vk::SurfaceFormatKHR>,
    present_modes: Vec<vk::PresentModeKHR>,
}

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Vulkan Triangle in Rust")
        .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
        .build(&event_loop)
        .expect("Failed to create window");

    let mut app = VulkanApp::new(&window);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::MainEventsCleared => unsafe {
                app.draw_frame();
            },
            _ => {}
        }
    });
}
