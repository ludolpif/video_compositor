use anyhow::Result;
use compositor_chromium::cef;
use compositor_common::{scene::Resolution, Framerate};
use log::{error, info};
use serde_json::json;
use std::{process::Command, thread, time::Duration};
use video_compositor::http;

use crate::common::write_example_sdp_file;

#[path = "./common/common.rs"]
mod common;

const VIDEO_RESOLUTION: Resolution = Resolution {
    width: 1920,
    height: 1080,
};
const FRAMERATE: Framerate = Framerate(30);

fn main() {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );
    ffmpeg_next::format::network::init();

    let target_path = &std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("..");

    if cef::bundle_app(target_path).is_err() {
        panic!("Build process helper first: cargo build --bin process_helper");
    }

    thread::spawn(|| {
        if let Err(err) = start_example_client_code() {
            error!("{err}")
        }
    });

    http::Server::new(8001).run();
}

fn start_example_client_code() -> Result<()> {
    thread::sleep(Duration::from_secs(2));

    info!("[example] Sending init request.");
    common::post(&json!({
        "type": "init",
        "framerate": FRAMERATE,
    }))?;

    info!("[example] Start listening on output port.");
    let output_sdp = write_example_sdp_file("127.0.0.1", 8002)?;
    Command::new("ffplay")
        .args(["-protocol_whitelist", "file,rtp,udp", &output_sdp])
        .spawn()?;

    info!("[example] Send register output request.");
    common::post(&json!({
        "type": "register",
        "entity_type": "output_stream",
        "output_id": "output_1",
        "port": 8002,
        "ip": "127.0.0.1",
        "resolution": {
            "width": VIDEO_RESOLUTION.width,
            "height": VIDEO_RESOLUTION.height,
        },
        "encoder_settings": {
            "preset": "ultrafast"
        }
    }))?;

    let shader_source = include_str!("../compositor_render/examples/silly/silly.wgsl");
    info!("[example] Register shader transform");
    common::post(&json!({
        "type": "register",
        "entity_type": "shader",
        "shader_id": "example_shader",
        "source": shader_source,
    }))?;

    info!("[example] Register web renderer transform");
    common::post(&json!({
        "type": "register",
        "entity_type": "web_renderer",
        "instance_id": "example_website",
        "url": "https://www.membrane.stream/", // or other way of providing source
        "resolution": { "width": VIDEO_RESOLUTION.width, "height": VIDEO_RESOLUTION.height },
    }))?;

    info!("[example] Update scene");
    common::post(&json!({
        "type": "update_scene",
        "nodes": [
           {
               "node_id": "shader_1",
               "type": "shader",
               "shader_id": "example_shader",
               "input_pads": [
                   "web_renderer_1",
               ],
               "resolution": { "width": VIDEO_RESOLUTION.width, "height": VIDEO_RESOLUTION.height },
           },
           {
               "node_id": "web_renderer_1",
               "type": "web_renderer",
               "instance_id": "example_website",
               "resolution": { "width": VIDEO_RESOLUTION.width, "height": VIDEO_RESOLUTION.height },
           }
        ],
        "outputs": [
            {
                "output_id": "output_1",
                "input_pad": "shader_1"
            }
        ]
    }))?;

    info!("[example] Start pipeline");
    common::post(&json!({
        "type": "start",
    }))?;

    Ok(())
}