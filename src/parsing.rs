use std::sync::Arc;

use anyhow::{anyhow, Result, Ok};
use ini::ini;

use crate::{bar_vertex, outline_vertex, Colour, Complex32, FourierSet};

use glium::{glutin::surface::WindowSurface, Program};

pub struct IniData<'a> {
    pub bg_colour: Colour,
    pub fps: f32,
    pub time: f32,
    pub lines: Vec<FourierSet<'a>>,
    pub render: bool,

    _bar_program: Arc<Program>,
    _outline_program: Arc<Program>,
}

macro_rules! get_expect {
    ($hash:ident, $($name:literal),+) => {
        ($($hash.get($name).map(|x| x.as_deref()).flatten().ok_or(anyhow!("Unable to find key {}", $name))?,)+)
    };
}



impl<'a> IniData<'a> {
    pub fn parse_ini(
        path: &str,
        facade: &glium::Display<WindowSurface>,
    ) -> Result<IniData<'a>> {
        let data = ini!(path);

        let setup = data
            .get("setup")
            .ok_or(anyhow!("Unable to find the Setup key"))?;

        let (bg_colour, fps, time, render) =
            get_expect!(setup, "bg_colour", "fps", "time", "render");

        let bg_colour = bg_colour.parse()?;
        let fps = fps.parse()?;
        let time = time.parse()?;
        let render = render.trim() == "yes";

        let mut output = IniData {
            bg_colour,
            lines: vec![],
            fps,
            time,
            render,
            _bar_program: Arc::new(bar_vertex::get_program(facade)),
            _outline_program: Arc::new(outline_vertex::get_program(facade)),
        };

        let bar_program = Arc::clone(&output._bar_program);
        let outline_program = Arc::clone(&output._outline_program);
        
        let mut lines = data
            .iter()
            .filter_map::<_, _>(|(key, inner)| {
                let bar_program = bar_program.clone();
                let outline_program = outline_program.clone();

                key.starts_with("line").then(|| {
                        let (
                            points,
                            samples,
                            outline_colour,
                            outline_width,
                            bar_colour,
                            bar_width,
                        ) = get_expect!(
                            inner,
                            "points",
                            "samples",
                            "outline_colour",
                            "outline_width",
                            "bar_colour",
                            "bar_width"
                        );
                        let outline_colour = outline_colour.parse()?;
                        let bar_colour = bar_colour.parse()?;

                        let mut points = points
                            .trim_start_matches("(")
                            .trim_end_matches(")")
                            .split("),(")
                            .map(|x| {
                                let mut split = x.split(",").map(|x| Ok(x.parse::<f32>()?));
                                Ok(Complex32::new(
                                    split.next().ok_or(anyhow!("Error parsing complex from {x}"))??, 
                                    split.next().ok_or(anyhow!("Error parsing complex from {x}"))??))
                            })
                            .collect::<Result<Vec<_>>>()?;

                        let samples = samples.parse()?;

                        let outline_width = outline_width.parse()?;
                        let bar_width = bar_width.parse()?;

                        Ok(FourierSet::new(
                            &mut points,
                            samples,
                            outline_colour,
                            outline_width,
                            outline_program,
                            bar_colour,
                            bar_width,
                            bar_program,
                            facade,
                        ))
                    })
            })
            .collect::<Result<Vec<_>>>()?;

        std::mem::swap(&mut output.lines, &mut lines);
        Ok(output)
    }
}
