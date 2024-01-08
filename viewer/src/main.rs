use raytracing_pipeline::RaytracingPipeline;

mod gui_renderer;
mod raytracing_pipeline;
mod vulkan_renderer;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).init();
    vulkan_renderer::VulkanBasicRenderer::<RaytracingPipeline>::new()
        .unwrap_or_else(|err| {
            log::error!("Error during initialization:\n{:?}", err);
            std::process::abort()
        })
        .run();
}
