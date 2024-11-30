mod canvas;
mod coord;
use std::io::Write;

use canvas::Canvas;
use coord::Coord;

/// strange number for no reason. Change if you need it.
const PINS: usize = 288;
/// Range: [0,1]
const LINE_WEIGHT: f32 = 20.0 / 256.0;
/// Distance between two pins we choose.
/// Make it smaller if you find a black ring around the center.
const MIN_DISTANCE: usize = 20;
/// How many lines we should calculate.
const MAX_LINES: usize = 4000;
/// How long should we omit a pin after it is used.
const TABU_SIZE: usize = 10;
/// This is independent with file you provide.
/// Higher value means more accurate but slower.
const IMG_SIZE: usize = 800;

fn main() {
    let file = std::env::args().skip(1).next().expect("Pls provide a file");
    eprintln!("Loading file...");
    let raw_image = load_image(std::fs::File::open(file).expect("Cannot open the file"));
    eprintln!("Preparing lines...");
    let pin_coords = calculate_pin_coords(IMG_SIZE);
    let line_cache = precalculate_all_potential_lines(&pin_coords);
    eprintln!("Drawing lines...");
    let line_sequence =
        calculate_lines(&Canvas::from_image(&raw_image, IMG_SIZE as _), &line_cache);

    let size = raw_image.width().min(raw_image.height());
    let mut final_image = Canvas::from_image(&raw_image, size);
    final_image.0.pixels_mut().for_each(|p| p[0] = 1.0);
    draw_lines(&mut final_image, &line_sequence);
    final_image.save_to_file("temp.png").unwrap();
    println!("{:?}", line_sequence);
}

fn load_image(mut file: std::fs::File) -> image::DynamicImage {
    use std::io::Read;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).expect("Failed to read file");
    image::load_from_memory(&buffer).expect("Not a valid image file")
}

fn calculate_pin_coords(img_size: usize) -> Vec<Coord> {
    let mut pin_coords = Vec::new();
    let center = img_size as f32 / 2.0;
    let radius = (img_size / 2) as f32 - 1.0;

    for i in 0..PINS {
        let angle = 2.0 * std::f32::consts::PI * i as f32 / PINS as f32;
        pin_coords.push(Coord {
            x: center + radius * angle.cos(),
            y: center + radius * angle.sin(),
        });
    }
    pin_coords
}
fn precalculate_all_potential_lines(pin_coords: &[Coord]) -> Vec<Vec<Vec<Coord>>> {
    let mut line_cache = vec![vec![vec![]; PINS]; PINS];

    for i in 0..PINS {
        for offset in MIN_DISTANCE..PINS - i {
            let start = pin_coords[i];
            let end = pin_coords[(i + offset) % PINS];
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

fn calculate_lines(source_image: &Canvas, line_cache: &Vec<Vec<Vec<Coord>>>) -> Vec<usize> {
    let mut error = source_image.clone();
    error.invert();
    let mut line_sequence = Vec::with_capacity(MAX_LINES);
    let mut recent_pins = std::collections::VecDeque::with_capacity(TABU_SIZE + 1);

    let mut current_pin = 0;
    line_sequence.push(0);

    for i in 0..MAX_LINES {
        if i & 511 == 0 {
            eprintln!("{i}/{MAX_LINES}");
            error.save_to_file("temp.png").unwrap();
        }
        let (_, next_ping, line) = (MIN_DISTANCE..PINS - MIN_DISTANCE)
            .filter_map(|offset| {
                let next_pin = (current_pin + offset) % PINS;
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
        if recent_pins.len() > TABU_SIZE {
            recent_pins.pop_back();
        }
        line_sequence.push(next_ping);
        current_pin = next_ping;

        line.iter().for_each(|point| {
            *error.get_pixel_mut(*point) -= LINE_WEIGHT;
        })
    }
    let mut result = source_image.clone();
    result.overlay(&error);
    result.save_to_file("temp.png").unwrap();

    line_sequence
}

fn draw_lines(canvas: &mut Canvas, line_sequence: &[usize]) {
    let pin_coords = calculate_pin_coords(canvas.0.width() as _);
    let mut pre = 0;
    for &pin in &line_sequence[1..] {
        lines_pace(pin_coords[pre], pin_coords[pin])
            .for_each(|point| *canvas.get_pixel_mut(point) -= LINE_WEIGHT);
        pre = pin;
    }
}
