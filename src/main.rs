#![deny(clippy::all)]
#![forbid(unsafe_code)]

use std::env;
use std::net::UdpSocket;
use std::time::Duration;
use error_iter::ErrorIter as _;
use log::{error};
use pixels::{Error, Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::{Event, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

const DEFAULT_WIDTH: usize = 48;
const DEFAULT_HEIGHT: usize = 24;
const DEFAULT_SCALE: usize = 24;
const DEFAULT_PORT: usize = 54321;
const RGB_SIZE: usize = 3;

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();

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
    let scale: usize = args
        .get(4)
        .and_then(|w| w.parse().ok())
        .unwrap_or(DEFAULT_SCALE);

    let address = format!("0.0.0.0:{}", port);
    let buffer_size = height * width * RGB_SIZE; // Calculate the buffer size dynamically
    let socket = UdpSocket::bind(&address).unwrap();

    println!("UDP server listening on {}", address);


    let window = {
        let size = LogicalSize::new(width as f64, height as f64);
        let scaled_size = LogicalSize::new(width as f64 * scale as f64, height as f64 * scale as f64);
        WindowBuilder::new()
            .with_title("Fun LED Simulator")
            .with_inner_size(scaled_size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(width as u32, height as u32, surface_texture)?
    };

    let mut buffer = vec![0u8; buffer_size];

    socket.set_read_timeout(Some(Duration::from_millis(10))).unwrap();

    let frame = pixels.frame_mut();
    for pixel in frame.chunks_exact_mut(4) {
        let rgba = [0, 0, 0, 0xFF];
        pixel.copy_from_slice(&rgba);
    }

    pixels.render()?;

    event_loop.run(move |event, _, control_flow| {

        if let Event::RedrawRequested(_) = event {
            if let Err(err) = pixels.render() {
                log_error("pixels.render", err);
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.close_requested() {
                *control_flow = ControlFlow::Exit;
                return;
            }
            window.request_redraw();
        }

        if let Event::MainEventsCleared = event {
            match socket.recv_from(&mut buffer) {
                Ok((_size, _src)) => {

                    // Convert the received data into a flat array of RGB tuples
                    let mut flat_array: Vec<(u8, u8, u8)> = vec![(0, 0, 0); height * width];

                    for i in 0..flat_array.len() {
                        let start_index = i * RGB_SIZE;
                        let red = buffer[start_index];
                        let green = buffer[start_index + 1];
                        let blue = buffer[start_index + 2];
                        flat_array[i] = (red, green, blue);
                    }

                    // Access the frame buffer of `pixels` and update pixel colors
                    let frame = pixels.frame_mut();
                    for (color, pixel) in flat_array.iter().zip(frame.chunks_exact_mut(4)) {
                        let (red, green, blue) = *color;
                        let rgba = [red, green, blue, 0xFF];
                        pixel.copy_from_slice(&rgba);
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // Timeout occurred, do nothing
                }
                Err(e) => {
                    // Handle other errors
                    eprintln!("Error receiving data: {}", e);
                }
            }

        }
    });
}

fn log_error<E: std::error::Error + 'static>(method_name: &str, err: E) {
    error!("{method_name}() failed: {err}");
    for source in err.sources().skip(1) {
        error!("  Caused by: {source}");
    }
}