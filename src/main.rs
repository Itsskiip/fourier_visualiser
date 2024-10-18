#![feature(fn_traits)]

#[macro_use]
extern crate glium;

use std::{
    f32::consts::PI,
    thread::sleep,
    time::{Duration, Instant},
};

use anyhow::{anyhow, Ok, Result};

use glium::{
    uniforms::{EmptyUniforms, UniformsStorage}, winit::window::Window, DrawParameters, Surface, VertexBuffer
};

use ini::ini;

#[derive(Copy, Clone, Default)]
struct Vertex {
    position: [f32; 2],
    alpha: f32,
}

impl std::fmt::Display for Vertex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.position[0], self.position[1])
    }
}

impl TryFrom<&str> for Vertex {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        let mut v = value.split(",").map(|x| Ok(x.trim().parse()?));
        let err = "Unable to parse value as a vertex";
        Ok(Self {
            position: [
                v.next().ok_or(anyhow!(err))??,
                v.next().ok_or(anyhow!(err))??,
            ],
            alpha: 1.0,
        })
    }
}

impl From<Vertex> for Complex<f32> {
    fn from(value: Vertex) -> Self {
        Complex {
            re: value.position[0],
            im: value.position[1],
        }
    }
}

impl From<Complex<f32>> for Vertex {
    fn from(value: Complex<f32>) -> Self {
        Vertex {
            position: [value.re, value.im],
            alpha: Default::default(),
        }
    }
}

#[derive(Clone, Copy)]
struct Colour {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

struct IniData<'a> {
    bg_colour: Colour,
    fps: f32,
    time: f32,
    lines: Vec<FourierSet<'a>>,
    render: bool,
}

impl From<Colour> for [f32; 4] {
    fn from(value: Colour) -> Self {
        [value.r, value.g, value.b, value.a]
    }
}

impl std::fmt::Display for Colour {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}, {}, {}, {}", self.r, self.g, self.b, self.a)
    }
}

impl From<&str> for Colour {
    fn from(value: &str) -> Self {
        let val: Vec<f32> = value
            .split(",")
            .map(|x| x.trim().parse().expect("Error parsing colour"))
            .collect();

        Self {
            r: val[0],
            g: val[1],
            b: val[2],
            a: val[3],
        }
    }
}

macro_rules! get_expect {
    ($hash:ident, $($name:literal),+) => {
        ($($hash.get($name).map(|x| x.as_deref()).flatten().ok_or(anyhow!("Unable to find key {}", $name))?,)+)
    };
}

implement_vertex!(Vertex, position);

impl IniData<'_> {
    fn parse_ini<'a>(
        path: &str,
        facade: &glium::Display<WindowSurface>,
    ) -> Result<IniData<'a>> {
        let data = ini!(path);

        let setup = data
            .get("setup")
            .ok_or(anyhow!("Unable to find the Setup key"))?;

        let (bg_colour, fps, time, render) =
            get_expect!(setup, "bg_colour", "fps", "time", "render");

        let bg_colour = Colour::from(bg_colour);
        let fps = fps.parse()?;
        let time = time.parse()?;
        let render = render == "yes";

        let lines = data
            .iter()
            .filter_map(|(key, inner)| {
                key.starts_with("line").then(|| {
                        let (
                            points,
                            samples,
                            outline_colour,
                            outline_width,
                            link_colour,
                            link_width,
                        ) = get_expect!(
                            inner,
                            "points",
                            "samples",
                            "outline_colour",
                            "outline_width",
                            "link_colour",
                            "link_width"
                        );
                        let outline_colour = Colour::from(outline_colour);
                        let link_colour = Colour::from(link_colour);

                        let mut points = points
                            .trim_start_matches("(")
                            .trim_end_matches(")")
                            .split("),(")
                            .map(|x| Vertex::try_from(x).unwrap())
                            .collect::<Vec<_>>();

                        let samples = samples.parse()?;

                        let outline_width = outline_width.parse()?;
                        let link_width = link_width.parse()?;

                        normalise(&mut points);
                        let links = fourier(&points);

                        Ok(FourierSet::new(
                            points,
                            samples,
                            outline_colour,
                            outline_width,
                            links,
                            link_colour,
                            link_width,
                            facade,
                        ))
                    })
            })
            .collect::<Result<Vec<_>>>();

        Ok(IniData {
            bg_colour,
            lines: lines?,
            fps,
            time,
            render,
        })
    }
}

struct FourierSet<'a> {
    points: Vec<Vertex>,
    outline_uniforms: glium::uniforms::UniformsStorage<'a, [f32; 4], EmptyUniforms>,
    outline_vbuffer: VertexBuffer<Vertex>,
    outline_buffer: Vec<Vertex>,
    outline_samples: usize,
    outline_cursor: usize,

    links: Vec<(i32, Complex<f32>)>,
    link_uniforms: glium::uniforms::UniformsStorage<'a, [f32; 4], EmptyUniforms>,
    link_buffer: VertexBuffer<Vertex>,
}

use glium::glutin::surface::WindowSurface;

impl<'a> FourierSet<'a> {
    fn new(
        points: Vec<Vertex>,
        samples: usize,
        outline_colour: Colour,
        outline_width: f32,

        links: Vec<(i32, Complex<f32>)>,
        link_colour: Colour,
        link_width: f32,

        facade: &glium::Display<WindowSurface>,
    ) -> Self {
        let link_buffer =
            VertexBuffer::dynamic(facade, &calculate_fourier(&fourier(&points), 0.0)).unwrap();

        let outline_vbuffer = VertexBuffer::empty_dynamic(facade, samples).unwrap();
        Self {
            points,
            outline_uniforms: uniform! {vertex_colour: outline_colour.into()},
            outline_vbuffer,
            outline_buffer: Vec::with_capacity(samples),
            outline_samples: samples,
            outline_cursor: 0,
            links,
            link_uniforms: uniform! {vertex_colour: link_colour.into()},
            link_buffer,
        }
    }
}

fn fourier(points: &[Vertex]) -> Vec<(i32, Complex<f32>)> {
    let result: Vec<(i32, _)> = (0..points.len() / 2)
        .map(|n| {
            let n = n as i32;
            if n == 0 {
                vec![n]
            } else {
                vec![n, -n]
            }
        })
        .flatten()
        .map(|n| (n, get_fourier_coef(points, n)))
        .collect();

    result
}

fn calculate_fourier(links: &[(i32, Complex<f32>)], t: f32) -> Vec<Vertex> {
    let mut prev = links[0].1.clone();
    let mut result = Vec::new();
    for (i, link) in links.iter() {
        let bar = prev
            + Complex::from_polar(
                link.norm(),
                link.arg() + (2.0 * PI * (f32::from(i16::try_from(*i).unwrap())) * t) as f32,
            );
        result.push(bar.into());
        prev = bar;
    }

    result
}

fn normalise(points: &mut [Vertex]) {
    let mut ranges = (0_f32..0_f32, 0_f32..0_f32);
    let mut center = (0_f32, 0_f32);

    for p in points.iter() {
        if p.position[0] > ranges.0.end {
            ranges.0.end = p.position[0]
        };
        if p.position[0] < ranges.0.start {
            ranges.0.start = p.position[0]
        };
        if p.position[1] > ranges.1.end {
            ranges.1.end = p.position[1]
        };
        if p.position[1] < ranges.1.start {
            ranges.1.start = p.position[1]
        };

        center.0 += p.position[0];
        center.1 += p.position[1];
    }

    center.0 /= points.len() as f32;
    center.1 /= points.len() as f32;

    let scale = 1.0
        / (ranges.0.end - ranges.0.start)
            .abs()
            .max((ranges.1.end - ranges.1.start).abs());

    for p in points {
        p.position[0] -= center.0;
        p.position[0] *= scale;
        p.position[1] -= center.1;
        p.position[1] *= scale;
    }
}

use num::Complex;

fn get_fourier_coef(points: &[Vertex], index: i32) -> Complex<f32> {
    let p_len = points.len() as f32;
    points
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let (r, mut theta) = Complex::from(*p).to_polar();
            theta += (-(i as f32) * 2.0 * PI * (index as f32)) / p_len;
            Complex::from_polar(r, theta)
        })
        .reduce(|a, b| a + b)
        .unwrap()
        .scale(1.0 / p_len)
}

fn get_linkage_program(display: &glium::Display<WindowSurface>) -> glium::Program {
    let vertex_shader_src = r#"
            #version 140

            in vec2 position;
            in float alpha;

            out float v_alpha;

            void main() {
                vec2 pos = position;
                gl_Position = vec4(pos, 0.0, 1.0);
            }
        "#;

        let fragment_shader_src = r#"
            #version 140

            uniform vec4 vertex_colour;
            in float v_alpha;
            out vec4 color;

            void main() {{
                color = vec4(vertex_colour.r, vertex_colour.g, vertex_colour.b, v_alpha);
            }}
        "#;

        
        glium::Program::from_source(display, vertex_shader_src, fragment_shader_src, None)
                .unwrap()
}

fn main() {
    let event_loop = glium::winit::event_loop::EventLoop::builder()
        .build()
        .expect("event loop building");

    let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new()
        .with_title("Fourier Series Visualiser")
        .build(&event_loop);

    let args = IniData::parse_ini("data.ini", &display).unwrap();

    let mut draw_lines = args.lines;
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::LineStrip);

    let linkage_program = get_linkage_program(&display);

    let mut t = 0_f32;

    let target_fps = args.fps;
    let inner_samples = target_fps * args.time;

    println!("{}", inner_samples);

    let target_ft = Duration::from_secs_f64(1.0 / f64::from(target_fps));

    let mut prev_frame = Instant::now();

    #[allow(deprecated)]
    event_loop
        .run(move |ev, window_target| match ev {
            glium::winit::event::Event::WindowEvent { event, .. } => match event {
                glium::winit::event::WindowEvent::CloseRequested => {
                    window_target.exit();
                }
                glium::winit::event::WindowEvent::RedrawRequested => {
                    let mut target = display.draw();

                    t = if t >= 1.0 {
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

                    for item in &mut draw_lines {
                        let link_buf = &item.link_buffer.as_mut_slice();
                        link_buf.write(&calculate_fourier(&item.links, t));

                        let cur = &mut item.outline_cursor;
                        let n = item.outline_samples;
                        while (*cur < n) && ((*cur as f32) / (n as f32)) <= t {
                            item.outline_buffer.push(
                                *calculate_fourier(&item.links, (*cur as f32) / (n as f32))
                                    .last()
                                    .unwrap(),
                            ); //maybe write a new algo
                            *cur += 1;
                        }

                        let outline_buf = &item.outline_vbuffer.as_mut_slice();
                        outline_buf.write(&item.outline_buffer);

                        target
                            .draw(
                                &item.link_buffer,
                                &indices,
                                &program,
                                &item.link_uniforms,
                                &DrawParameters::default(),
                            )
                            .unwrap();
                        target
                            .draw(
                                &item.outline_vbuffer,
                                &indices,
                                &program,
                                &item.outline_uniforms,
                                &DrawParameters::default(),
                            )
                            .unwrap();
                    }

                    target.finish().unwrap();
                }
                glium::winit::event::WindowEvent::Resized(window_size) => {
                    display.resize(window_size.into());
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
                    window.request_redraw();
                }
            }
            _ => (),
        })
        .unwrap();
}
