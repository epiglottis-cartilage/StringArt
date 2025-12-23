mod canvas;
use canvas::Canvas;
mod genetic;
mod tabu;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(version)]
struct Args {
    /// path to the image file.
    #[clap()]
    file: std::path::PathBuf,
    /// number of pins to draw.
    /// number for no reason
    #[clap(long, default_value_t = 288)]
    pin: usize,
    /// how many lines should calculate.
    #[clap(long, default_value_t = 4000)]
    lines: usize,
    /// weight of the lines.
    /// Range: [0,1]
    #[clap( long, default_value_t = 20./256.)]
    line_weight: f32,
    /// size of the image.
    /// bigger means more accurate but slower.
    #[clap(long, default_value_t = 800)]
    img_size: u32,
    /// distance between two pins we choose.
    #[clap(long, default_value_t = 20)]
    distance: usize,
    /// how long should we omit a pin after it is used.
    #[clap(long, default_value_t = 10)]
    tabu: usize,

    #[clap(long, default_value_t = 100)]
    population_size: usize,
    /// probability of crossover
    /// Range: [0,1]
    #[clap(long, default_value_t = 0.8)]
    crossover_rate: f32,
    /// probability of mutation
    /// Range: [0,1]
    #[clap(long, default_value_t = 0.1)]
    mutation_rate: f32,
    /// number of generations iterate
    #[clap(long, default_value_t = 100)]
    generations: usize,

    /// output file name
    #[clap(short, long, default_value = "output.png")]
    output: std::path::PathBuf,
}

fn main() {
    let args = Args::parse();

    let raw_image = Canvas::from(
        &image::open(&args.file).expect("file is invalid"),
        args.img_size,
    );

    let pin_coords = utils::calculate_pin_coords(&args);
    let line_cache = utils::precalculate_all_potential_lines(&raw_image, &pin_coords, &args);

    let line_sequence = genetic::calculate_lines(&raw_image, &line_cache, &args);

    println!("{:?}", line_sequence);
    let final_image: image::DynamicImage =
        utils::draw_lines(&line_sequence, &line_cache, &args).into();

    final_image
        .to_luma8()
        .save(args.output)
        .expect("Failed to save the file");
}

mod utils {
    use crate::{Args, Canvas};
    use glam::Vec2;
    use rayon::prelude::*;

    pub fn calculate_pin_coords(args: &Args) -> Vec<Vec2> {
        let mut pin_coords = Vec::new();
        let center = args.img_size as f32 / 2.0;
        let radius = (args.img_size / 2) as f32 - 1.0;

        for i in 0..args.pin {
            let angle = 2.0 * std::f32::consts::PI * i as f32 / args.pin as f32;
            pin_coords.push(Vec2 {
                x: center + radius * angle.cos(),
                y: center + radius * angle.sin(),
            });
        }
        pin_coords
    }
    pub fn precalculate_all_potential_lines(
        img: &Canvas,
        pin_coords: &[Vec2],
        args: &Args,
    ) -> Vec<Vec<Vec<Vec2>>> {
        // let mut line_cache = vec![vec![vec![]; args.pin]; args.pin];
        (0..args.pin)
            .into_par_iter()
            .map(|i| {
                (0..args.pin)
                    .into_par_iter()
                    .map(|j| {
                        let start = pin_coords[i];
                        let end = pin_coords[j];
                        img.line_space_coord(start, end)
                    })
                    .collect()
            })
            .collect()
    }
    pub fn draw_lines(
        line_sequence: &[usize],
        line_cache: &Vec<Vec<Vec<Vec2>>>,
        args: &Args,
    ) -> Canvas {
        let mut canvas = Canvas::new(args.img_size, 1.0);
        line_sequence.windows(2).for_each(|l| {
            let [s, e] = *l else { unreachable!() };
            let (s, e) = if s < e { (s, e) } else { (e, s) };
            line_cache[s][e].iter().for_each(|point| {
                *canvas.get_pixel_mut(*point) -= args.line_weight;
            })
        });
        canvas
    }
}
