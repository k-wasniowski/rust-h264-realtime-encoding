use std::sync::mpsc::{Receiver, Sender};
use crate::camera::CameraInfo;
use std::thread;
use log::{debug, error};
use openh264::encoder::{EncodedBitStream, EncoderConfig};
use openh264::encoder::Encoder;
use openh264::formats::YUVBuffer;
use openh264::OpenH264API;
use crate::video_frame;

pub fn run(
    camera_info: CameraInfo,
    video_source: Receiver<video_frame::VideoFrame>,
    encoded_video_sender: Sender<video_frame::EncodedVideoFrame>,
) {
    let width = camera_info.width;
    let height = camera_info.height;
    let config = EncoderConfig::new(width as u32, height as u32);
    config.enable_skip_frame(true);
    config.set_bitrate_bps(10);
    config.debug(true);

    let api = OpenH264API::from_source();
    let mut encoder = match Encoder::with_config(api, config) {
        Ok(encoder) => encoder,
        Err(e) => panic!("Failed to create encoder - {}", e),
    };

    thread::spawn(move || {
        loop {
            let frame = match video_source.recv() {
                Ok(frame) => frame,
                Err(e) => {
                    error!("Failed to receive video frame - {}", e);
                    continue;
                }
            };

            let mut buffer = YUVBuffer::new(width as usize, height as usize);
            buffer.read_rgb(&frame.buffer);

            let encoded_bitstream: EncodedBitStream = match encoder.encode(&buffer) {
                Ok(x) => x,
                Err(e) => {
                    println!("Failed to encode - {}", e);
                    continue;
                }
            };

            let encoded_video_frame = video_frame::EncodedVideoFrame {
                buffer: encoded_bitstream.to_vec(),
                timestamp: frame.timestamp,
            };

            match encoded_video_sender.send(encoded_video_frame) {
                Ok(_) => debug!("Successfully sent frame to the encoded video sink"),
                Err(e) => error!("Failed to send frame to the encoded video sink - {}", e)
            }
        }
    });
}