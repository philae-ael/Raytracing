use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use tev_client::TevClient;
use tev_renderer::TevRenderer;

mod tev_renderer;

#[derive(Parser, Debug)]
struct Args {
    tev_path: Option<PathBuf>,
    #[arg(short, long)]
    size: Option<String>,
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let args = Args::parse();
    let client = if let Some(tev_path) = args.tev_path {
        let command = std::process::Command::new(tev_path);
        TevClient::spawn(command)
    } else {
        Ok(TevClient::wrap(std::net::TcpStream::connect(
            "127.0.0.1:14158",
        )?))
    }?;
    TevRenderer {
        width: 800,
        height: 600,
        tile_size: 20,
    }
    .run(client)
    .unwrap();
    Ok(())
}
