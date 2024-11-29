mod coord;
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
const IMG_SIZE: usize = 600;
/// I don't what it is :)
const FILE_PATH: &str = "./pic/pp.png";

fn main() {
    println!("Loading file...");
    let source_image =
        image_to_pixel_array(std::fs::File::open(FILE_PATH).expect("Failed to open file"));
    println!("Preparing lines...");
    let pin_coords = calculate_pin_coords();
    let line_cache = precalculate_all_potential_lines(&pin_coords);
    println!("Drawing lines...");
    let results = calculate_lines(&source_image, &line_cache);

    println!("\n{:?}", results);
}

fn save_temp_file(buffer: &[f32]) {
    use image::{ImageBuffer, Luma};

    let mut image_buffer = ImageBuffer::new(IMG_SIZE as _, IMG_SIZE as _);
    for (x, y, pixel) in image_buffer.enumerate_pixels_mut() {
        let idx = (y * IMG_SIZE as u32 + x) as usize;
        let value = (buffer[idx] * 255.0) as u8;
        *pixel = Luma([value]);
    }

    image_buffer.save("temp.png").unwrap();
    // std::thread::sleep(std::time::Duration::from_micros(100));
}
fn image_to_pixel_array(mut file: std::fs::File) -> Vec<f32> {
    use std::io::Read;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).expect("Failed to read file");

    let mut image = image::load_from_memory(&buffer).expect("Failed to load image");
    let (w, h) = (image.width(), image.height());
    let size = w.min(h);
    let image = image.crop((w - size) / 2, (h - size) / 2, size, size);
    let image = image.resize(
        IMG_SIZE as _,
        IMG_SIZE as _,
        image::imageops::FilterType::Lanczos3,
    );
    image
        .to_rgba32f()
        .pixels()
        .map(|pixel| {
            let [r, g, b, a] = pixel.0;
            let (r, g, b) = (
                r * a + r * (1.0 - a),
                g * a + g * (1.0 - a),
                b * a + b * (1.0 - a),
            );
            let luma = r * 0.3 + g * 0.5 + b * 0.2;
            luma.min(0.9)
        })
        .collect()
}
fn calculate_pin_coords() -> Vec<Coord> {
    let mut pin_coords = Vec::new();
    let center = IMG_SIZE as f32 / 2.0;
    let radius = (IMG_SIZE / 2) as f32 - 1.0;

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
            line_cache[i][offset] = lines_pace(start, end);
        }
    }

    line_cache
}

fn lines_pace(start: Coord, end: Coord) -> Vec<Coord> {
    let n = start.distance(end) as usize;
    let step = Coord {
        x: (end.x - start.x) / (n as f32 - 1.0),
        y: (end.y - start.y) / (n as f32 - 1.0),
    };
    (0..n).map(|i| (start + step * i as f32).round()).collect()
}

fn calculate_lines(source_image: &Vec<f32>, line_cache: &Vec<Vec<Vec<Coord>>>) -> Vec<usize> {
    let mut error = source_image
        .iter()
        .copied()
        .map(|pixel| (1.0 - pixel))
        .collect::<Vec<_>>();
    let mut line_sequence = Vec::with_capacity(MAX_LINES);
    let mut recent_pins = std::collections::VecDeque::with_capacity(TABU_SIZE + 1);

    fn err_alone_line(err: &Vec<f32>, line: &Vec<Coord>) -> f32 {
        line.iter()
            .map(|point| {
                let v = (point.y * IMG_SIZE as f32) + point.x;
                err[v as usize]
            })
            .sum()
    }

    let mut current_pin = 0;
    line_sequence.push(0);

    for i in 0..MAX_LINES {
        if i & 511 == 0 {
            println!("{i}/{MAX_LINES}");
            save_temp_file(&error);
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
                    err_alone_line(&error, &line_cache[start][offset]),
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

        for point in line {
            let v = (point.y * IMG_SIZE as f32) + point.x;
            error[v as usize] -= LINE_WEIGHT;
        }
    }
    let string_image = source_image
        .iter()
        .copied()
        .zip(error.iter().copied())
        .map(|(s, e)| s + e)
        .collect::<Vec<_>>();
    save_temp_file(&string_image);
    line_sequence
}
