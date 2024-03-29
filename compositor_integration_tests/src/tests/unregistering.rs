use std::{thread, time::Duration};

use crate::{
    compare_video_dumps, input_dump_from_disk, output_dump_from_disk, CommunicationProtocol,
    CompositorInstance, OutputReceiver, PacketSender,
};
use anyhow::Result;
use serde_json::json;

pub fn unregistering() -> Result<()> {
    let instance = CompositorInstance::start(8020);

    register_output_with_initial_scene(&instance).expect_err("Image has to be registered first");

    instance.send_request(json!({
        "type": "register",
        "entity_type": "image",
        "asset_type": "svg",
        "image_id": "image_1",
        "url": "https://compositor.live/img/logo.svg"
    }))?;

    register_output_with_initial_scene(&instance)?;

    let output_receiver = OutputReceiver::start(
        8021,
        CommunicationProtocol::Tcp,
        Duration::from_secs(10),
        "unregistering_test_output.rtp",
    )?;

    instance.send_request(json!({
        "type": "register",
        "entity_type": "rtp_input_stream",
        "transport_protocol": "udp",
        "input_id": "input_1",
        "port": 8022,
        "video": {
            "codec": "h264"
        },
    }))?;

    let input_1_dump = input_dump_from_disk("8_colors_input_video.rtp")?;
    let mut input_1_sender = PacketSender::new(CommunicationProtocol::Udp, 8022)?;

    instance.send_request(json!({
        "type": "start",
    }))?;

    thread::sleep(Duration::from_secs(2));

    input_1_sender.send(&input_1_dump)?;
    instance.send_request(json!({
        "type": "unregister",
        "entity_type": "input_stream",
        "input_id": "input_1",
    }))?;

    instance.send_request(json!({
        "type": "unregister",
        "entity_type": "image",
        "image_id": "image_1",
    }))?;

    let new_output_dump = output_receiver.wait_for_output()?;
    let output_dump_from_disk = output_dump_from_disk("unregistering_test_output.rtp")?;

    compare_video_dumps(
        &output_dump_from_disk,
        &new_output_dump,
        &[Duration::from_secs(1), Duration::from_secs(3)],
        20.0,
    )?;

    Ok(())
}

fn register_output_with_initial_scene(instance: &CompositorInstance) -> Result<()> {
    instance.send_request(json!({
        "type": "register",
        "entity_type": "output_stream",
        "output_id": "output_1",
        "transport_protocol": "tcp_server",
        "port": 8021,
        "video": {
            "resolution": {
                "width": 1280,
                "height": 720,
            },
            "encoder_preset": "medium",
            "initial": {
                "type": "tiles",
                "padding": 3,
                "background_color_rgba": "#DDDDDDFF",
                "children": [
                    {
                        "type": "input_stream",
                        "input_id": "input_1",
                    },
                    {
                        "type": "image",
                        "image_id": "image_1",
                    },
                ],
            }
        },
    }))
}