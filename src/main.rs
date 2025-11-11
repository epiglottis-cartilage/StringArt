mod canvas;

use std::fs::File;

use canvas::Canvas;
use clap::Parser;
use glam::Vec2;
use rayon::prelude::*;

#[derive(Parser, Debug)]
#[clap(version)]
struct Args {
    /// path to the image file.
    #[clap()]
    file: std::path::PathBuf,
    /// number of pins to draw.
    /// number for no reason
    #[clap(short, long, default_value_t = 288)]
    pin: usize,
    /// how many lines should calculate.
    #[clap(short, long, default_value_t = 4000)]
    lines: usize,
    /// weight of the lines.
    /// Range: [0,1]
    #[clap(short, long, default_value_t = 20./256.)]
    weight: f32,
    /// size of the image.
    /// bigger means more accurate but slower.
    #[clap(long, default_value_t = 800)]
    size: u32,

    /// distance between two pins we choose.
    #[clap(long, default_value_t = 20)]
    distance: usize,
    /// how long should we omit a pin after it is used.
    #[clap(long, default_value_t = 10)]
    tabu: usize,

    /// output file name
    #[clap(short, long, default_value = "output.png")]
    output: std::path::PathBuf,
}

fn main() {
    let args = Args::parse();
    let file = &load_image(File::open(&args.file).expect("Cannot open the file"));
    let raw_image = Canvas::from(file, args.size);
    let pin_coords = calculate_pin_coords(&args);
    let line_cache = precalculate_all_potential_lines(&raw_image, &pin_coords, &args);

    let line_sequence = calculate_lines(&raw_image, &line_cache, &args);

    let mut final_image = Canvas::new(args.size, 1.0);
    draw_lines(&mut final_image, &line_sequence, &line_cache, &args);
    println!("{:?}", line_sequence);
    let final_image: image::DynamicImage = final_image.into();
    final_image
        .to_luma8()
        .save(args.output)
        .expect("Failed to save the file");
}

fn load_image(mut file: std::fs::File) -> image::DynamicImage {
    use std::io::Read;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).expect("Failed to read file");
    image::load_from_memory(&buffer).expect("Not a valid image file")
}

fn calculate_pin_coords(args: &Args) -> Vec<Vec2> {
    let mut pin_coords = Vec::new();
    let center = args.size as f32 / 2.0;
    let radius = (args.size / 2) as f32 - 1.0;

    for i in 0..args.pin {
        let angle = 2.0 * std::f32::consts::PI * i as f32 / args.pin as f32;
        pin_coords.push(Vec2 {
            x: center + radius * angle.cos(),
            y: center + radius * angle.sin(),
        });
    }
    pin_coords
}
fn precalculate_all_potential_lines(
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

fn calculate_lines(
    source_image: &Canvas,
    line_cache: &Vec<Vec<Vec<Vec2>>>,
    args: &Args,
) -> Vec<usize> {
    let mut error = source_image.clone();
    error.invert();
    let mut line_sequence = Vec::with_capacity(args.lines);
    let mut recent_pins = std::collections::VecDeque::with_capacity(args.tabu + 1);

    let mut current_pin = 0;
    line_sequence.push(0);

    for i in 0..args.lines {
        if i & 511 == 0 {
            eprintln!("{i}/{}", args.lines);
        }
        let (_, next_ping, line) = (args.distance..args.pin - args.distance)
            .par_bridge()
            .filter_map(|offset| {
                let next_pin = (current_pin + offset) % args.pin;
                if recent_pins.contains(&next_pin) {
                    return None;
                }

                let (start, end) = if current_pin < next_pin {
                    (current_pin, next_pin)
                } else {
                    (next_pin, current_pin)
                };

                Some((
                    line_cache[start][end]
                        .iter()
                        .map(|point| error.get_pixel(*point))
                        .sum::<f32>(),
                    next_pin,
                    &line_cache[start][end],
                ))
            })
            .max_by(|(err1, _, _), (err2, _, _)| err1.total_cmp(err2))
            .unwrap();

        recent_pins.push_front(next_ping);
        if recent_pins.len() > args.tabu {
            recent_pins.pop_back();
        }
        line_sequence.push(next_ping);
        current_pin = next_ping;

        line.iter().for_each(|point| {
            *error.get_pixel_mut(*point) -= args.weight;
        })
    }

    line_sequence
}

fn draw_lines(
    canvas: &mut Canvas,
    line_sequence: &[usize],
    line_cache: &Vec<Vec<Vec<Vec2>>>,
    args: &Args,
) {
    line_sequence.windows(2).for_each(|l| {
        let [s, e] = *l else { unreachable!() };
        let (s, e) = if s < e { (s, e) } else { (e, s) };
        line_cache[s][e].iter().for_each(|point| {
            *canvas.get_pixel_mut(*point) -= args.weight;
        })
    });
}
