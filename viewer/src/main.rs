use raytracing_pipeline::RaytracingPipeline;

mod gui_renderer;
mod raytracing_pipeline;
mod vulkan_renderer;

fn main() {
    let env = env_logger::Env::default()
        .filter("LOG_LEVEL")
        .default_filter_or("trace");

    env_logger::Builder::new()
        .filter_module("vulkan::validation", log::LevelFilter::Off)
        .parse_env(env)
        .init();

    vulkan_renderer::VulkanBasicRenderer::<RaytracingPipeline>::new()
        .unwrap_or_else(|err| {
            log::error!("Error during initialization:\n{:?}", err);
            std::process::abort()
        })
        .run();
}
