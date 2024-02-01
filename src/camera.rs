use std::sync::mpsc::Sender;
use log::{debug, error, info, warn};
use opencv::{prelude::*, videoio};
use std::thread;
use std::time::SystemTime;
use crate::video_frame;

pub struct CameraInfo {
    pub width: usize,
    pub height: usize,
    pub framerate: u32,
}

pub fn run(video_sink: Sender<video_frame::VideoFrame>) -> CameraInfo {
    let mut camera = match videoio::VideoCapture::new(0, videoio::CAP_ANY) {
        Ok(camera) => camera,
        Err(e) => panic!("Failed to create video capture module - {}", e),
    };

    match camera.is_opened() {
        Ok(_) => info!("Successfully opened camera stream"),
        Err(e) => panic!("Failed to open camera stream with error: {}", e),
    };

    let width = match camera.get(videoio::CAP_PROP_FRAME_WIDTH) {
        Ok(width) => width as usize,
        Err(e) => panic!("Failed to get camera frame width with error: {}", e),
    };

    let height = match camera.get(videoio::CAP_PROP_FRAME_HEIGHT) {
        Ok(height) => height as usize,
        Err(e) => panic!("Failed to get camera frame height with error: {}", e),
    };

    let framerate = match camera.get(videoio::CAP_PROP_FPS) {
        Ok(framerate) => framerate as u32,
        Err(e) => panic!("Failed to get camera framerate with error: {}", e),
    };

    thread::spawn(move || loop {
        let mut frame = Mat::default();

        let frame_grabbed = match camera.read(&mut frame) {
            Ok(result) => result,
            Err(e) => {
                error!("Failed to read frame - {}", e);
                continue;
            }
        };

        if !frame_grabbed {
            warn!("Camera frame read failed!");
            continue;
        }

        let mut rgb_frame = Mat::default();
        match opencv::imgproc::cvt_color(&frame, &mut rgb_frame, opencv::imgproc::COLOR_BGR2RGB, 0) {
            Ok(_) => {},
            Err(e) => {
                error!("Failed to convert frame color - {}", e);
                continue;
            }
        };

        let frame_timestamp = SystemTime::now();

        let frame_data = match rgb_frame.data_bytes() {
            Ok(frame_data) => frame_data,
            Err(e) => {
                error!("Failed to get frame data - {}", e);
                continue;
            }
        };

        let frame_data = frame_data.to_vec();

        let video_frame = video_frame::VideoFrame::new(frame_timestamp, frame_data);

        let result = video_sink.send(video_frame);
        match result {
            Ok(_) => debug!("Successfully sent frame to the video sink"),
            Err(e) => error!("Failed to send frame to the video sink - {}", e)
        };
    });

    CameraInfo {
        width,
        height,
        framerate,
    }
}
