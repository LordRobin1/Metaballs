#![allow(unused)]
#![allow(dead_code)]

// use colored::*;
use softbuffer::GraphicsContext;
use std::time::{Duration, Instant};
use winit::dpi::PhysicalSize;
use winit::event::{Event, WindowEvent, WindowEvent::CursorMoved};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use ColorMode::*;

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut graphics_context = unsafe { GraphicsContext::new(&window, &window) }.unwrap();
    let mut fps: u128 = 0;
    let mut start = Instant::now();
    let mut delta_time = Default::default();
    let mut last_len = 0;

    let mut cursor = Point { x: 0, y: 0 };

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::MainEventsCleared => {
                start = Instant::now();

                let size = window.inner_size();
                render(&mut graphics_context, &size, &cursor);

                // FPS
                delta_time = start.elapsed();
                print!("\r");
                // last_len = fps.to_string().len();
                fps = 1_000_000 / delta_time.as_micros();
                print!(
                    "FPS: {}, Cursor: {}, {}{}",
                    fps,
                    cursor.x,
                    cursor.y,
                    " ".repeat(50),
                );
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                window_id,
                event:
                    CursorMoved {
                        device_id,
                        position,
                        ..
                    },
            } if window_id == window.id() => {
                (cursor.x, cursor.y) = (position.x as u32, position.y as u32);
            }
            _ => {}
        }
    });
}

fn render(context: &mut GraphicsContext, size: &PhysicalSize<u32>, cursor: &Point) {
    let mut buffer = vec![0; (size.width * size.height) as usize];
    shaders(&mut buffer, size, cursor);
    context.set_buffer(&buffer, size.width as u16, size.height as u16);
}

fn shaders(buffer: &mut [u32], size: &PhysicalSize<u32>, cursor: &Point) {
    let (width, height) = (size.width, size.height);
    let mut pxl: Pixel;
    let circ_1 = cursor;
    let mid = Point {
        x: size.width / 2,
        y: size.height / 2,
    };
    let circ_2 = &Point {
        x: mid.x + 50,
        y: mid.y,
    };
    let circ_3 = &Point {
        x: mid.x - 50,
        y: mid.y,
    };
    // let thickness = 10.0;
    let radius = 100.0;
    let r = 0xff0000;
    let g = 0xff00;
    let b = 0xff;

    for y in 0..height {
        for x in 0..width {
            pxl = Pixel {
                pos: Point { x, y },
                color: 0x00,
            };
            circle_shader(&mut pxl, circ_1, &radius, &r, ColorMode::Lerp);
            circle_shader(&mut pxl, circ_2, &radius, &g, ColorMode::Lerp);
            circle_shader(&mut pxl, circ_3, &radius, &b, ColorMode::Lerp);
            // ring_shader(&mut pxl, &ring, &radius, &thickness, ColorMode::Additive);
            // ring_shader(&mut pxl, &ring1, &radius, &thickness, ColorMode::Additive);

            buffer[(y * width + x) as usize] = pxl.color;
        }
    }
}

fn ring_shader(
    pxl: &mut Pixel,
    center: &Point,
    radius: &f32,
    thickness: &f32,
    col_mode: ColorMode,
) {
    let distance = center.distance(&pxl.pos);

    let in_circle = smooth_step(distance, *radius, *radius - 3.);
    let in_ring = 1. - smooth_step(distance, (*radius - thickness), (*radius - thickness) - 3.);

    let rgb = (255.0 * in_circle * in_ring) as u32;

    let mut color = rgb;
    color = (color << 8) + rgb;
    color = (color << 8) + rgb;

    match col_mode {
        Lerp => pxl.color = color_lerp(color, pxl.color, 0.5),
        Additive => pxl.color = color_add(pxl.color, color),
        _ => (),
    }
}

fn circle_shader(pxl: &mut Pixel, center: &Point, radius: &f32, color: &u32, col_mode: ColorMode) {
    let distance = center.distance(&pxl.pos);

    let in_circle = smooth_step(distance, *radius, *radius - 3.);

    match col_mode {
        Lerp => pxl.color = color_lerp(pxl.color, *color, in_circle),
        Additive => pxl.color = color_add(pxl.color, *color),
        _ => (),
    }
}

fn step(value: f32, edge: f32) -> f32 {
    match value < edge {
        true => 1.,
        _ => 0.,
    }
}

/// returns a value between 0. and 1.
fn smooth_step(value: f32, edge_0: f32, edge_1: f32) -> f32 {
    let x = ((value - edge_0) / (edge_1 - edge_0)).clamp(0., 1.);
    x * x * (3. - 2. * x)
}

fn color_lerp(color_0: u32, color_1: u32, weight: f32) -> u32 {
    match (color_0, color_1) {
        (0, 0) => 0,
        _ => {
            let (red_0, green_0, blue_0) = to_rgb(color_0);
            let (red_1, green_1, blue_1) = to_rgb(color_1);

            let red_lp = (red_0 + weight * (red_1 - red_0)) as u32;
            let green_lp = (green_0 + weight * (green_1 - green_0)) as u32;
            let blue_lp = (blue_0 + weight * (blue_1 - blue_0)) as u32;

            (red_lp << 16) + (green_lp << 8) + blue_lp
        }
    }
}

fn color_add(color_0: u32, color_1: u32) -> u32 {
    match (color_0, color_1) {
        (0, 0) => 0,
        (0, _) => color_1,
        (_, 0) => color_0,
        _ => {
            let red_add = (((color_0 >> 16) & 0xff) + ((color_1 >> 16) & 0xff)).clamp(0, 255);
            let green_add = (((color_0 >> 8) & 0xff) + ((color_1 >> 8) & 0xff)).clamp(0, 255);
            let blue_add = ((color_0 & 0xff) + (color_1 & 0xff)).clamp(0, 255);

            (red_add << 16) + (green_add << 8) + blue_add
        }
    }
}

fn to_rgb(color: u32) -> (f32, f32, f32) {
    let red = (color >> 16) & 0xff;
    let green = (color >> 8) & 0xff;
    let blue = color & 0xff;

    (red as f32, green as f32, blue as f32)
}

// not adapted to new shader system yet
fn dist_to_center(pxl: &Pixel, width: &u32, height: &u32) -> f32 {
    let origin = Point { x: 0, y: 0 };
    let mid = Point {
        x: *width / 2,
        y: *height / 2,
    };
    let max_dist = origin.distance(&mid);

    let distance = mid.distance(&pxl.pos);
    distance / max_dist
}

#[derive(Debug)]
struct Pixel {
    pos: Point,
    color: u32,
}

#[derive(Debug)]
struct Point {
    x: u32,
    y: u32,
}

impl Point {
    fn distance(&self, point: &Point) -> f32 {
        let d_x = self.x as f32 - point.x as f32;
        let d_y = self.y as f32 - point.y as f32;
        (d_x * d_x + d_y * d_y).sqrt()
    }
    fn in_range(&self, point: &Point, range: f32) -> bool {
        let d_x = self.x as f32 - point.x as f32;
        let d_y = self.y as f32 - point.y as f32;
        (d_x * d_x + d_y * d_y) < range.powf(2.)
    }
}

enum ColorMode {
    Lerp,
    Additive,
}
