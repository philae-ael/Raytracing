use std::sync::Arc;

use vulkano::{
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
        PrimaryAutoCommandBuffer, PrimaryCommandBuffer, RenderPassBeginInfo, SubpassContents,
    },
    device::{
        physical::{PhysicalDevice, PhysicalDeviceType},
        Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo,
    },
    image::{view::ImageView, ImageAccess, ImageUsage, SwapchainImage},
    instance::{debug::DebugUtilsLabel, Instance, InstanceCreateInfo},
    pipeline::graphics::viewport::Viewport,
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass},
    swapchain::{
        acquire_next_image, AcquireError, Surface, Swapchain, SwapchainAbstract,
        SwapchainCreateInfo, SwapchainCreationError, SwapchainPresentInfo,
    },
    sync::{self, FlushError, GpuFuture},
    VulkanLibrary,
};
use vulkano_win::VkSurfaceBuild;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use anyhow::{Context, Result};

pub trait CustomPipeline
where
    Self: Sized,
{
    fn setup(
        render_pass: Arc<RenderPass>,
        device: Arc<Device>,
        queue: Arc<Queue>,
        dimensions: [u32; 2],
    ) -> Result<Self>;

    fn on_resize(&mut self, dimensions: [u32; 2]) -> Result<()>;

    fn prerun(&mut self) {}

    fn render(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<
            PrimaryAutoCommandBuffer,
            StandardCommandBufferAllocator,
        >,
    ) -> Result<()>;

    fn uploads(&mut self) -> Option<Arc<PrimaryAutoCommandBuffer>>;
}

pub struct VulkanBasicRenderer<T: CustomPipeline + 'static> {
    event_loop: Option<EventLoop<()>>,
    surface: Arc<Surface<Window>>,
    device: Arc<Device>,
    queue: Arc<Queue>,
    swapchain: Arc<Swapchain<Window>>,
    render_pass: Arc<RenderPass>,
    viewport: Viewport,
    framebuffers: Vec<Arc<Framebuffer>>,

    recreate_swapchain: bool,

    app: T,
}

impl<T: CustomPipeline> VulkanBasicRenderer<T> {
    pub fn new() -> Result<Self> {
        let library = VulkanLibrary::new()?;
        let mut required_extensions = vulkano_win::required_extensions(&*library);
        required_extensions.ext_debug_utils = true;
        let instance = Instance::new(
            library,
            InstanceCreateInfo {
                enabled_extensions: required_extensions,
                // enable enumerating devices that use non-conformant vulkan implementations. (ex. moltenvk)
                enumerate_portability: true,
                ..Default::default()
            },
        )?;

        let event_loop = EventLoop::new();

        let surface = WindowBuilder::new().build_vk_surface(&event_loop, instance.clone())?;

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };

        let (physical_device, queue_family_index) =
            Self::select_gpu(instance.clone(), surface.clone(), &device_extensions)?;
        log::info!(
            "Using device {} (type {:?})",
            physical_device.properties().device_name,
            physical_device.properties().device_type
        );

        let (device, mut queues) = Device::new(
            physical_device,
            DeviceCreateInfo {
                enabled_extensions: device_extensions,
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                ..Default::default()
            },
        )?;

        let queue = queues.next().context("Could not find a queue")?;
        let (swapchain, images) = {
            let surface_capabilities = device
                .physical_device()
                .surface_capabilities(&surface, Default::default())?;

            // TODO: What are the available color formats ?
            let image_format = Some(
                device
                    .physical_device()
                    .surface_formats(&surface, Default::default())?[0]
                    .0,
            );

            Swapchain::new(
                device.clone(),
                surface.clone(),
                SwapchainCreateInfo {
                    min_image_count: surface_capabilities.min_image_count + 1,
                    image_format,
                    image_extent: surface.window().inner_size().into(),
                    image_usage: ImageUsage {
                        color_attachment: true,
                        ..ImageUsage::empty()
                    },
                    composite_alpha: surface_capabilities
                        .supported_composite_alpha
                        .iter()
                        .next()
                        .context("Did not find any composite alpha capability for surface")?,
                    ..Default::default()
                },
            )?
        };

        let render_pass = vulkano::ordered_passes_renderpass!(
            device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: swapchain.image_format(),
                    samples: 1,
                }
            },
            passes: [
                { color: [color], depth_stencil: {}, input: [] }
                // { color: [color], depth_stencil: {}, input: [] }
            ]
        )?;

        let viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: [0.0, 0.0],
            depth_range: 0.0..1.0,
        };

        let app = T::setup(
            render_pass.clone(),
            device.clone(),
            queue.clone(),
            images[0].dimensions().width_height()
        )?;

        let mut this = Self {
            event_loop: Some(event_loop),
            surface,
            device,
            queue,
            swapchain,
            render_pass,
            recreate_swapchain: false,
            viewport,
            framebuffers: vec![],
            app,
        };

        this.window_size_dependent_setup(&images)?;
        Ok(this)
    }

    fn select_gpu(
        instance: Arc<Instance>,
        surface: Arc<Surface<Window>>,
        device_extensions: &DeviceExtensions,
    ) -> Result<(Arc<PhysicalDevice>, u32)> {
        // Select the best GPU
        instance
            .enumerate_physical_devices()?
            .filter(|p| p.supported_extensions().contains(device_extensions))
            .filter_map(|p| {
                p.queue_family_properties()
                    .iter()
                    .enumerate()
                    .position(|(i, q)| {
                        q.queue_flags.graphics
                            && p.surface_support(i as u32, &surface).unwrap_or(false)
                    })
                    .map(|i| (p, i as u32))
            })
            .min_by_key(|(p, _)| match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                PhysicalDeviceType::Other => 4,
                _ => 5,
            })
            .context("Could not find a GPU")
    }

    pub fn run(mut self) {
        self.app.prerun();
        let event_loop = self.event_loop.take().unwrap();

        event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {
                self.recreate_swapchain = true;
            }
            Event::RedrawEventsCleared => {
                self.render_frame().unwrap_or_else(|err| -> () {
                    log::error!("{:#?}", err);
                    std::process::abort()
                });
            }
            _ => (),
        });
    }

    fn window_size_dependent_setup(
        &mut self,
        images: &[Arc<SwapchainImage<Window>>],
    ) -> Result<()> {
        let dimensions = images[0].dimensions().width_height();
        self.viewport.dimensions = [dimensions[0] as f32, dimensions[1] as f32];


        self.framebuffers = images
            .iter()
            .map(|image| -> Result<_> {
                let view = ImageView::new_default(image.clone())?;
                Ok(Framebuffer::new(
                    self.render_pass.clone(),
                    FramebufferCreateInfo {
                        attachments: vec![view],
                        ..Default::default()
                    },
                )?)
            })
            .collect::<Result<Vec<_>>>()?;
        self.app.on_resize(dimensions)?;
        Ok(())
    }

    fn render_frame(&mut self) -> Result<()> {
        let dimensions = self.surface.window().inner_size();
        if dimensions.width == 0 || dimensions.height == 0 {
            return Ok(());
        }

        if self.recreate_swapchain {
            let (new_swapchain, new_images) = match self.swapchain.recreate(SwapchainCreateInfo {
                image_extent: dimensions.into(),
                ..self.swapchain.create_info()
            }) {
                Ok(r) => r,
                Err(SwapchainCreationError::ImageExtentNotSupported { .. }) => return Ok(()),
                Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
            };

            self.swapchain = new_swapchain;
            self.window_size_dependent_setup(&new_images)?;
            self.recreate_swapchain = false;

        }
        let (image_index, suboptimal, acquire_future) =
            match acquire_next_image(self.swapchain.clone(), None) {
                Ok(r) => Ok(r),
                Err(AcquireError::OutOfDate) => {
                    self.recreate_swapchain = true;
                    return Ok(());
                }
                Err(e) => Err(e).context(format!("Failed to acquire next image: '{:?}'", e)),
            }?;

        if suboptimal {
            self.recreate_swapchain = true;
        }

        let command_buffer_allocator = StandardCommandBufferAllocator::new(self.device.clone());

        let mut builder = AutoCommandBufferBuilder::primary(
            &command_buffer_allocator,
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )?;

        builder.begin_debug_utils_label(DebugUtilsLabel {
            label_name: "Main pass".to_owned(),
            ..Default::default()
        })?;

        let builder_ref = builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([1.0, 0.47, 0.65, 1.0].into())],
                    ..RenderPassBeginInfo::framebuffer(
                        self.framebuffers[image_index as usize].clone(),
                    )
                },
                SubpassContents::Inline,
            )?
            .set_viewport(0, [self.viewport.clone()]);

        self.app.render(builder_ref)?;

        builder.end_render_pass()?;

        let command_buffer = builder.build()?;

        let uploads_future = if let Some(uploads_command_buffer) = self.app.uploads() {
            uploads_command_buffer.execute(self.queue.clone())?.boxed()
        } else {
            sync::now(self.device.clone()).boxed()
        };
        let future = uploads_future
            .join(acquire_future)
            .then_execute(self.queue.clone(), command_buffer)?
            .then_swapchain_present(
                self.queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(self.swapchain.clone(), image_index),
            )
            .then_signal_fence_and_flush();

        match future {
            Ok(_future) => Ok(()),
            Err(FlushError::OutOfDate) => {
                self.recreate_swapchain = true;
                Ok(())
            }
            Err(e) => Err(e),
        }?;
        Ok(())
    }
}
