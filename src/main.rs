#![feature(fn_traits)]
#![feature(iter_collect_into)]
#![feature(vec_push_within_capacity)]

#[macro_use]
extern crate glium;

use std::{
    process::exit,
    thread::sleep,
    time::{Duration, Instant},
};

mod bar_vertex;
use bar_vertex::BarVertex;

mod outline_vertex;
use outline_vertex::OutlineVertex;

mod colour;
use colour::Colour;

mod parsing;
use parsing::IniData;

mod fourier;
use fourier::FourierSet;

mod graphics;

mod buffer;

use glium::{backend::glutin::SimpleWindowBuilder, winit::event_loop::EventLoop, Surface};

use num::complex::Complex32;

fn main() {
    let program_start = Instant::now();
    let event_loop = EventLoop::new().unwrap();

    let (window, facade) = SimpleWindowBuilder::new()
        .with_title("Fourier Series Visualiser")
        .with_inner_size(720, 720)
        .build(&event_loop);

    let mut args = IniData::parse_ini("data.ini", &facade).unwrap();

    let mut t = 0_f32;

    let inner_samples = args.fps * args.time - 1.0;

    let target_ft = Duration::from_secs_f32(1.0 / (args.fps));

    let mut prev_frame = Instant::now();

    let mut iters = 0_u64;

    println!("Rendering {} animated frames, {} total outline positions and {} intermediate bar positions.", inner_samples, args.lines.iter().map(|l| l.outline_buffer.size).reduce(|a, b| a + b).unwrap(), args.lines.iter().map(|l| l.outline_buffer.size * l.bars.len()).reduce(|a, b| a + b).unwrap());
    let render_start = Instant::now();

    #[allow(deprecated)]
    event_loop
        .run(move |ev, window_target| match ev {
            glium::winit::event::Event::WindowEvent { event, .. } => match event {
                glium::winit::event::WindowEvent::CloseRequested => {
                    window_target.exit();
                }
                glium::winit::event::WindowEvent::RedrawRequested => {
                    let mut target = facade.draw();

                    t = if t > 1.0 {
                        iters += 1;
                        0.0
                    } else {
                        t + 1.0 / (inner_samples as f32)
                    };

                    target.clear_color(
                        args.bg_colour.r,
                        args.bg_colour.g,
                        args.bg_colour.b,
                        args.bg_colour.a,
                    );

                    for item in &mut args.lines {
                        item.draw(&mut target, t);
                    }

                    target.finish().unwrap();
                }
                glium::winit::event::WindowEvent::Resized(window_size) => {
                    facade.resize(window_size.into());
                }
                _ => (),
            },
            glium::winit::event::Event::AboutToWait => {
                if !args.render {
                    let now = Instant::now();
                    let elapsed = now - prev_frame;
                    if elapsed <= (target_ft) {
                        sleep(target_ft - elapsed);
                    }
                    window.request_redraw();
                    prev_frame = now;
                } else {
                    if iters > 1 {
                        let now = Instant::now();
                        println!(
                            "Time elapsed since program start: {}",
                            (now - program_start).as_secs_f32()
                        );
                        println!(
                            "Time elapsed since rendering start: {}",
                            (now - render_start).as_secs_f32()
                        );
                        exit(0);
                    }
                    window.request_redraw();
                }
            }
            _ => (),
        })
        .unwrap();
}
