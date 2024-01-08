use raytracing_pipeline::RaytracingPipeline;



mod vulkan_renderer;
mod raytracing_pipeline;
mod gui_renderer;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    vulkan_renderer::VulkanBasicRenderer::<RaytracingPipeline>::new()
        .unwrap_or_else(|err| {
            log::error!("Error during initialization:\n{:?}", err);
            std::process::abort()
        })
        .run();
}