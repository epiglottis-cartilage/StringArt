mod canvas;
mod coord;

use std::fs::File;

use canvas::Canvas;
use clap::Parser;
use coord::Coord;

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
    size: usize,

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
    let raw_image = load_image(File::open(&args.file).expect("Cannot open the file"));
    let pin_coords = calculate_pin_coords(args.size, &args);
    let line_cache = precalculate_all_potential_lines(&pin_coords, &args);

    let line_sequence = calculate_lines(
        &Canvas::from_image(&raw_image, args.size as _),
        &line_cache,
        &args,
    );

    let size = raw_image.width().min(raw_image.height()) * 4;
    let mut final_image = Canvas::from_image(&raw_image, size);
    final_image.0.pixels_mut().for_each(|p| p[0] = 1.0);
    draw_lines(&mut final_image, &line_sequence, &args);
    println!("{:?}", line_sequence);
    final_image
        .save_to_file(&args.output)
        .expect("Failed to save the file");
}

fn load_image(mut file: std::fs::File) -> image::DynamicImage {
    use std::io::Read;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).expect("Failed to read file");
    image::load_from_memory(&buffer).expect("Not a valid image file")
}

fn calculate_pin_coords(img_size: usize, args: &Args) -> Vec<Coord> {
    let mut pin_coords = Vec::new();
    let center = img_size as f32 / 2.0;
    let radius = (img_size / 2) as f32 - 1.0;

    for i in 0..args.pin {
        let angle = 2.0 * std::f32::consts::PI * i as f32 / args.pin as f32;
        pin_coords.push(Coord {
            x: center + radius * angle.cos(),
            y: center + radius * angle.sin(),
        });
    }
    pin_coords
}
fn precalculate_all_potential_lines(pin_coords: &[Coord], args: &Args) -> Vec<Vec<Vec<Coord>>> {
    let mut line_cache = vec![vec![vec![]; args.pin]; args.pin];

    for i in 0..args.pin {
        for offset in args.distance..args.pin - i {
            let start = pin_coords[i];
            let end = pin_coords[(i + offset) % args.pin];
            line_cache[i][offset] = lines_pace(start, end).collect();
        }
    }
    line_cache
}

fn lines_pace(start: Coord, end: Coord) -> impl DoubleEndedIterator<Item = Coord> {
    let n = start.distance(end) as usize;
    (0..=n)
        .map(move |i| (start * (1.0 - i as f32 / n as f32) + end * (i as f32 / n as f32)).round())
}

fn calculate_lines(
    source_image: &Canvas,
    line_cache: &Vec<Vec<Vec<Coord>>>,
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
                let offset = end - start;

                Some((
                    line_cache[start][offset]
                        .iter()
                        .map(|point| error.get_pixel(*point))
                        .sum::<f32>(),
                    next_pin,
                    &line_cache[start][offset],
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
    let mut result = source_image.clone();
    result.overlay(&error);

    line_sequence
}

fn draw_lines(canvas: &mut Canvas, line_sequence: &[usize], args: &Args) {
    let pin_coords = calculate_pin_coords(canvas.0.width() as _, args);
    let mut pre = 0;
    for &pin in &line_sequence[1..] {
        lines_pace(pin_coords[pre], pin_coords[pin])
            .for_each(|point| *canvas.get_pixel_mut(point) -= args.weight);
        pre = pin;
    }
}
