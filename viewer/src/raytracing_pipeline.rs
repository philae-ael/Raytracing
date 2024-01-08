use std::sync::{mpsc::Receiver, Arc};

use bytemuck::{Pod, Zeroable};
use image::{ImageBuffer, Rgba};
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer, TypedBufferAccess},
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
        CopyBufferToImageInfo, PrimaryAutoCommandBuffer,
    },
    descriptor_set::{
        allocator::StandardDescriptorSetAllocator, PersistentDescriptorSet, WriteDescriptorSet,
    },
    device::{Device, Queue},
    format::Format,
    image::{view::ImageView, ImageDimensions, StorageImage},
    impl_vertex,
    pipeline::{
        graphics::{
            color_blend::ColorBlendState,
            input_assembly::{InputAssemblyState, PrimitiveTopology},
            vertex_input::BuffersDefinition,
            viewport::ViewportState,
        },
        GraphicsPipeline, Pipeline, PipelineBindPoint,
    },
    render_pass::{RenderPass, Subpass},
    sampler::{Filter, Sampler, SamplerAddressMode, SamplerCreateInfo},
};

use crate::{
    gui_renderer::{GUIRenderer, PixelMsg},
    vulkan_renderer::CustomPipeline,
};
use anyhow::{Context, Result};

pub struct RaytracingPipeline {
    device: Arc<Device>,
    queue: Arc<Queue>,

    pipeline: Arc<GraphicsPipeline>,
    set: Arc<PersistentDescriptorSet>,

    vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>,

    image_buffer: Arc<CpuAccessibleBuffer<[f32]>>,
    uploads_command_buffer: Arc<PrimaryAutoCommandBuffer>,

    command_buffer_allocator: StandardCommandBufferAllocator,
    descriptor_set_allocator: StandardDescriptorSetAllocator,

    gui_renderer: GUIRenderer,
    channel: Option<Receiver<PixelMsg>>,
    size: [u32; 2],
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod)]
struct Vertex {
    position: [f32; 2],
}
impl_vertex!(Vertex, position);

impl RaytracingPipeline {
    fn window_size_dependent_setup(
        device: Arc<Device>,
        queue: Arc<Queue>,
        command_buffer_allocator: &StandardCommandBufferAllocator,
        descriptor_set_allocator: &StandardDescriptorSetAllocator,
        pipeline: Arc<GraphicsPipeline>,
        width: u32,
        height: u32,
    ) -> Result<(
        Arc<CpuAccessibleBuffer<[f32]>>,
        PrimaryAutoCommandBuffer,
        Arc<PersistentDescriptorSet>,
        GUIRenderer,
    )> {
        let mut uploads_image_from_cpu = AutoCommandBufferBuilder::primary(
            command_buffer_allocator,
            queue.queue_family_index(),
            CommandBufferUsage::MultipleSubmit,
        )?;

        let (image_buffer, texture) = {
            let dimensions = ImageDimensions::Dim2d {
                width,
                height,
                array_layers: 1,
            };

            let image_gpu = StorageImage::new(
                device.clone(),
                dimensions,
                Format::R32G32B32A32_SFLOAT,
                [queue.queue_family_index()],
            )?;

            // SAFETY: probably, if width and height and both non null
            let image_cpu = unsafe {
                let image_cpu = CpuAccessibleBuffer::<[f32]>::uninitialized_array(
                    device.clone(),
                    std::mem::size_of::<Rgba<f32>>() as u64 * width as u64 * height as u64,
                    BufferUsage {
                        transfer_src: true,
                        ..BufferUsage::empty()
                    },
                    false,
                )?;

                image_cpu.write()?.fill(0.0);
                image_cpu
            };
            uploads_image_from_cpu.copy_buffer_to_image(CopyBufferToImageInfo::buffer_image(
                image_cpu.clone(),
                image_gpu.clone(),
            ))?;

            (image_cpu, ImageView::new_default(image_gpu)?)
        };

        let sampler = Sampler::new(
            device.clone(),
            SamplerCreateInfo {
                mag_filter: Filter::Linear,
                min_filter: Filter::Linear,
                address_mode: [SamplerAddressMode::Repeat; 3],
                ..Default::default()
            },
        )?;

        let layout = pipeline.layout().set_layouts().get(0).unwrap();
        let set = PersistentDescriptorSet::new(
            descriptor_set_allocator,
            layout.clone(),
            [WriteDescriptorSet::image_view_sampler(0, texture, sampler)],
        )?;

        Ok((
            image_buffer,
            uploads_image_from_cpu.build()?,
            set,
            GUIRenderer::new(width, height),
        ))
    }

    fn run_renderer(&mut self) {
        let (tx, rx) = std::sync::mpsc::channel();

        // Note: if prerun has already been run, then
        self.channel = Some(rx);
        let mut gui_renderer = self.gui_renderer;
        std::thread::spawn(move || {
            gui_renderer.run(tx);
        });
    }
}

impl CustomPipeline for RaytracingPipeline {
    fn setup(
        render_pass: Arc<RenderPass>,
        device: Arc<Device>,
        queue: Arc<Queue>,
        dimensions: [u32; 2],
    ) -> Result<Self> {
        let [width, height] = dimensions;

        let vertices = [
            Vertex {
                position: [-1.0, 1.0],
            },
            Vertex {
                position: [-1.0, -1.0],
            },
            Vertex {
                position: [1.0, 1.0],
            },
            Vertex {
                position: [1.0, -1.0],
            },
        ];

        let vertex_buffer = CpuAccessibleBuffer::<[Vertex]>::from_iter(
            device.clone(),
            BufferUsage {
                vertex_buffer: true,
                ..BufferUsage::empty()
            },
            false,
            vertices,
        )?;

        let vs = vs::load(device.clone()).unwrap();
        let fs = fs::load(device.clone()).unwrap();

        let subpass = Subpass::from(render_pass.clone(), 0).unwrap();
        let pipeline = GraphicsPipeline::start()
            .vertex_input_state(BuffersDefinition::new().vertex::<Vertex>())
            .vertex_shader(
                vs.entry_point("main")
                    .context("Did not find entry point in vertex shader")?,
                (),
            )
            .input_assembly_state(
                InputAssemblyState::new().topology(PrimitiveTopology::TriangleStrip),
            )
            .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
            .fragment_shader(
                fs.entry_point("main")
                    .context("Did not find entry point in fragment shader")?,
                (),
            )
            .color_blend_state(ColorBlendState::new(subpass.num_color_attachments()).blend_alpha())
            .render_pass(subpass)
            .build(device.clone())?;

        // Image data

        let command_buffer_allocator = StandardCommandBufferAllocator::new(device.clone());
        let descriptor_set_allocator = StandardDescriptorSetAllocator::new(device.clone());
        let (image_buffer, uploads_command_buffer, set, gui_renderer) =
            Self::window_size_dependent_setup(
                device.clone(),
                queue.clone(),
                &command_buffer_allocator,
                &descriptor_set_allocator,
                pipeline.clone(),
                width,
                height,
            )?;

        Ok(Self {
            device,
            queue,
            vertex_buffer,
            pipeline,
            set,
            uploads_command_buffer: Arc::new(uploads_command_buffer),
            gui_renderer,
            image_buffer,
            command_buffer_allocator,
            descriptor_set_allocator,
            channel: None,
            size: [width, height],
        })
    }

    fn on_resize(&mut self, dimensions: [u32; 2]) -> Result<()> {
        let [width, height] = dimensions;

        let (image_buffer, uploads_command_buffer, set, gui_renderer) =
            Self::window_size_dependent_setup(
                self.device.clone(),
                self.queue.clone(),
                &self.command_buffer_allocator,
                &self.descriptor_set_allocator,
                self.pipeline.clone(),
                width,
                height,
            )?;
        self.image_buffer = image_buffer;
        self.uploads_command_buffer = Arc::new(uploads_command_buffer);
        self.set = set;
        self.size = dimensions;

        self.gui_renderer = gui_renderer;
        drop(self.channel.take());
        self.run_renderer();

        Ok(())
    }

    fn render(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<
            PrimaryAutoCommandBuffer,
            StandardCommandBufferAllocator,
        >,
    ) -> Result<()> {
        builder
            .bind_pipeline_graphics(self.pipeline.clone())
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.pipeline.layout().clone(),
                0,
                self.set.clone(),
            )
            .bind_vertex_buffers(0, self.vertex_buffer.clone())
            .draw(self.vertex_buffer.len() as u32, 1, 0, 0)?;
        Ok(())
    }

    fn uploads(&mut self) -> Option<Arc<PrimaryAutoCommandBuffer>> {
        let im = self.image_buffer.clone();
        let mut image_buffer: ImageBuffer<Rgba<f32>, _> = ImageBuffer::from_raw(
            self.size[0],
            self.size[1],
            im.write().expect(
                "This should never block? Who is using this buffer without my authorization?",
            ),
        )
        .expect("Image buffer and image size dont correspond");

        if self.channel.is_none() {
            self.run_renderer();
        }

        for PixelMsg { x, y, color } in self
            .channel
            .as_mut()
            .expect("This is unexepected? No renderer channel")
            .try_iter()
        {
            *image_buffer.get_pixel_mut(x, y) = color;
        }
        Some(self.uploads_command_buffer.clone())
    }
}

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
#version 450
layout(location = 0) in vec2 position;
layout(location = 0) out vec2 tex_coords;
void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    tex_coords = 0.5*(position + vec2(1.0));
}"
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
#version 450
layout(location = 0) in vec2 tex_coords;
layout(location = 0) out vec4 f_color;
layout(set = 0, binding = 0) uniform sampler2D tex;
void main() {
    f_color = texture(tex, tex_coords);
}"
    }
}
