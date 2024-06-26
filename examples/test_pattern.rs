use anyhow::Result;
use live_compositor::{server, types::Resolution};
use log::{error, info};
use serde::Deserialize;
use serde_json::json;
use std::{env, thread};

use crate::common::{start_ffplay, start_websocket_thread, stream_ffmpeg_testsrc};

#[path = "./common/common.rs"]
mod common;

const VIDEO_RESOLUTION: Resolution = Resolution {
    width: 1920,
    height: 1080,
};

const IP: &str = "127.0.0.1";
const OUTPUT_PORT: u16 = 8002;

fn main() {
    env::set_var("LIVE_COMPOSITOR_WEB_RENDERER_ENABLE", "0");
    ffmpeg_next::format::network::init();

    thread::spawn(|| {
        if let Err(err) = start_example_client_code() {
            error!("{err}")
        }
    });

    server::run()
}

#[derive(Deserialize)]
struct RegisterResponse {
    port: u16,
}

fn start_example_client_code() -> Result<()> {
    info!("[example] Start listening on output port.");
    start_ffplay(IP, OUTPUT_PORT, None)?;
    start_websocket_thread();

    info!("[example] Send register input request.");
    let RegisterResponse { port: input_port } = common::post(
        "input/input_1/register",
        &json!({
            "type": "rtp_stream",
            "port": "8004:8008",
            "video": {
                "decoder": "ffmpeg_h264"
            },
            "offset_ms": 0,
            "required": true,
        }),
    )?
    .json::<RegisterResponse>()?;

    let shader_source = include_str!("./silly.wgsl");
    info!("[example] Register shader transform");
    common::post(
        "shader/example_shader/register",
        &json!({
            "source": shader_source,
        }),
    )?;

    let scene = json!( {
        "type": "shader",
        "id": "shader_1",
        "shader_id": "example_shader",
        "children": [
            {
                "type": "input_stream",
                "input_id": "input_1",
            }
        ],
        "resolution": { "width": VIDEO_RESOLUTION.width, "height": VIDEO_RESOLUTION.height },
    });

    info!("[example] Send register output request.");
    common::post(
        "output/output_1/register",
        &json!({
            "type": "rtp_stream",
            "port": OUTPUT_PORT,
            "ip": "127.0.0.1",
            "video": {
                "resolution": {
                    "width": VIDEO_RESOLUTION.width,
                    "height": VIDEO_RESOLUTION.height,
                },
                "encoder": {
                    "type": "ffmpeg_h264",
                    "preset": "ultrafast"
                },
                "initial": {
                    "root": scene
                }
            }
        }),
    )?;

    info!("[example] Start input stream");
    stream_ffmpeg_testsrc(IP, input_port, VIDEO_RESOLUTION)?;

    info!("[example] Start pipeline");
    common::post("start", &json!({}))?;

    Ok(())
}
