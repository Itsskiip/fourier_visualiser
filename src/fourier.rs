use glium::{glutin::surface::WindowSurface, Display, Frame};

use std::{f32::consts::PI, sync::Arc};

use crate::{
    bar_vertex::Bar, buffer::Buffer, graphics::Drawable, outline_vertex::Outline, BarVertex, Colour, Complex32, OutlineVertex
};

pub struct FourierSet<'a> {
    n: usize, //number of points given in the input, also equivalent to the number of bars

    pub outline_gpu: Outline<'a>,
    pub outline_buffer: Buffer<OutlineVertex>,

    pub bars: Vec<(i32, Complex32)>,
    pub bar_gpu: Bar<'a>,
}

impl<'a> FourierSet<'a> {
    pub fn new(
        points: &mut [Complex32],
        samples: usize,

        outline_colour: Colour,
        outline_width: f32,
        outline_program: Arc<glium::Program>,

        bar_colour: Colour,
        bar_width: f32,
        bar_program: Arc<glium::Program>,

        facade: &glium::Display<WindowSurface>,
    ) -> Self {
        let n = points.len();
        normalise(points);
        let bars = fourier_transform(&points);

        Self {
            n,
            outline_gpu: Outline::new(facade, samples, outline_program, outline_colour),
            outline_buffer: Buffer::new(samples),
            bars,
            bar_gpu: Bar::new(facade, n, bar_program, bar_colour),
        }
    }

    pub fn draw(&mut self, facade: &mut Frame, t: f32) {
        self.bar_gpu.upload(&self.get_bar_pos(t));
        self.bar_gpu.draw(facade).unwrap();
        
        while self.outline_buffer.has_capacity() && (self.outline_buffer.percent_full() < t) {
            self.calc_next_bar_pos();
        };

        let slice = self.outline_buffer.as_full_slice();
        
        self.outline_gpu.upload(slice);
        self.outline_gpu.draw(facade).unwrap();
    }

    pub fn get_bar_pos(&self, t: f32) -> Vec<BarVertex> {
        let result:Vec<BarVertex> = get_bar_pos_iter(&self.bars, t).map(|c| c.into()).collect();
        result
    }

    pub fn calc_next_bar_pos(&mut self) {
        let val = get_bar_pos_iter(&self.bars, self.outline_buffer.percent_full()).last().unwrap().into();
        self.outline_buffer.push(val);
    }
}

fn get_bar_pos_iter(
    bars: &[(i32, Complex32)],
    t: f32,
) -> impl Iterator<Item = Complex32> + use<'_> {
    bars.iter()
        .map(move |(rot, cur)| {
            Complex32::from_polar(
                cur.norm(),
                cur.arg() + (2.0 * PI * f32::from(i16::try_from(*rot).unwrap()) * t),
            )
        })
        .scan(Complex32::ZERO, |state, new| {
            *state += new;
            Some(*state)
        })
}

fn fourier_transform(points: &[Complex32]) -> Vec<(i32, Complex32)> {
    let mut result: Vec<(i32, _)> = (0..=points.len() / 2)
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

    if result.len() > points.len() {
        result.pop();
    }
    
    result
}

fn normalise(points: &mut [Complex32]) {
    let mut ranges = (0_f32..0_f32, 0_f32..0_f32);
    //let mut center = Complex32::ZERO;

    for p in points.iter() {
        if p.re > ranges.0.end {
            ranges.0.end = p.re
        };
        if p.re < ranges.0.start {
            ranges.0.start = p.re
        };
        if p.im > ranges.1.end {
            ranges.1.end = p.im
        };
        if p.im < ranges.1.start {
            ranges.1.start = p.im
        };

        //center += p;
    }

    //center = center.scale(1.0 / points.len() as f32);

    let scale = 1.0
        / (ranges.0.end - ranges.0.start)
            .abs()
            .max((ranges.1.end - ranges.1.start).abs());

    for p in points {
        //*p -= center;
        *p = p.scale(scale);
    }
}

fn get_fourier_coef(points: &[Complex32], index: i32) -> Complex32 {
    let p_len = points.len() as f32;
    points
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let (r, mut theta) = p.to_polar();
            theta += (-(i as f32) * 2.0 * PI * (index as f32)) / p_len;
            Complex32::from_polar(r, theta)
        })
        .reduce(|a, b| a + b)
        .unwrap()
        .scale(1.0 / p_len)
}
