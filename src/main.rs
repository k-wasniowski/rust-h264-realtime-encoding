mod camera;
mod video_frame;
mod encoder;
mod logging;

use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::time::SystemTime;
use log::{debug, error, info};

fn main() {
    logging::initialize();

    let (sender, receiver): (Sender<video_frame::VideoFrame>, Receiver<video_frame::VideoFrame>) = mpsc::channel();

    let camera_info = camera::run(sender);

    info!("Camera resolution: {}x{}", camera_info.width, camera_info.height);

    let (encoded_video_sender, encoded_video_receiver): (Sender<video_frame::EncodedVideoFrame>, Receiver<video_frame::EncodedVideoFrame>) = mpsc::channel();

    encoder::run(camera_info, receiver, encoded_video_sender);

    loop {
        let encoded_video_frame = match encoded_video_receiver.recv() {
            Ok(encoded_video_frame) => encoded_video_frame,
            Err(e) => {
                error!("Failed to receive encoded video frame - {}", e);
                continue;
            }
        };

        let current_time = SystemTime::now();

        let age = current_time.duration_since(encoded_video_frame.timestamp).unwrap();

        debug!("Received encoded video frame with age: {}", age.as_millis());
    }
}