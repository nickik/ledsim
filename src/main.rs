#![deny(clippy::all)]
#![forbid(unsafe_code)]

use error_iter::ErrorIter as _;
use log::error;
use pixels::{Pixels, SurfaceTexture};
use std::env;
use std::net::UdpSocket;
use winit::{
    dpi::LogicalSize,
    event_loop::EventLoop,
    window::WindowBuilder,
};

const DEFAULT_WIDTH: usize = 48;
const DEFAULT_HEIGHT: usize = 24;
const DEFAULT_PORT: usize = 54321;
const RGB_SIZE: usize = 3;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let port: usize = args
        .get(1)
        .and_then(|p| p.parse().ok())
        .unwrap_or(DEFAULT_PORT);
    let height: usize = args
        .get(2)
        .and_then(|h| h.parse().ok())
        .unwrap_or(DEFAULT_HEIGHT);
    let width: usize = args
        .get(3)
        .and_then(|w| w.parse().ok())
        .unwrap_or(DEFAULT_WIDTH);

    let buffer_size = height * width * RGB_SIZE; // Calculate the buffer size dynamically
    let address = format!("127.0.0.1:{}", port);

    let socket = UdpSocket::bind(&address)?;
    println!("UDP server listening on {}", address);

    let mut buffer = vec![0u8; buffer_size]; // Dynamically sized buffer
    let event_loop = EventLoop::new().unwrap();

    let window = {
        let size = LogicalSize::new(width as f64, height as f64);
        let scaled_size = LogicalSize::new(width as f64 * 12.0, height as f64 * 12.0);
        WindowBuilder::new()
            .with_title("Fun LED Simulator")
            .with_inner_size(scaled_size)
            .with_min_inner_size(size)
            .build(&event_loop) // Pass the `event_loop` directly, not a `Result`
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(width as u32, height as u32, surface_texture).unwrap()
    };

    if let Err(err) = pixels.render() {
        log_error("pixels.render", err);
        panic!()
    }

    // Receive data
    loop {
        let (size, src) = socket.recv_from(&mut buffer)?;
        println!("Received {} bytes from {}", size, src);

        if size != buffer_size {
            println!(
                "Unexpected data size: expected {} bytes, got {} bytes",
                buffer_size, size
            );
            continue;
        }

        // Convert the received data into a flat array of RGB tuples
        let mut flat_array: Vec<(u8, u8, u8)> = vec![(0, 0, 0); height * width];

        for i in 0..flat_array.len() {
            let start_index = i * RGB_SIZE;
            let red = buffer[start_index];
            let green = buffer[start_index + 1];
            let blue = buffer[start_index + 2];
            flat_array[i] = (red, green, blue);
        }

        println!("Converted data into a flat array of RGB colors:");

        // Access the frame buffer of `pixels` and update pixel colors
        let frame = pixels.frame_mut();
        for (color, pixel) in flat_array.iter().zip(frame.chunks_exact_mut(4)) {
            let (red, green, blue) = *color;
            let rgba = [red, green, blue, 0xFF];
            pixel.copy_from_slice(&rgba);
        }

        if let Err(err) = pixels.render() {
            log_error("pixels.render", err);
            panic!()
        }
    }
}

fn log_error<E: std::error::Error + 'static>(method_name: &str, err: E) {
    error!("{method_name}() failed: {err}");
    for source in err.sources().skip(1) {
        error!("  Caused by: {source}");
    }
}