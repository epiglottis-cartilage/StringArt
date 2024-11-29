use std::fs::File;
use std::io::Read;
use std::time::Instant;

const PINS: usize = 288; // strange number for no reason
const MIN_DISTANCE: usize = 20;
const MAX_LINES: usize = 4000;
const LINE_WEIGHT: f64 = 20.0 / 255.0;
const IMG_SIZE: usize = 500; // this is independent with file you provide.
const FILE_PATH: &str = "not-human.jpg";

struct Coord {
    x: f64,
    y: f64,
}

fn main() {
    let start_time = Instant::now();
    println!("Loading file...");
    let source_image = image_to_pixel_array(File::open(FILE_PATH).expect("Failed to open file"));
    save_temp_file(&source_image);
    println!("Preparing lines...");
    let pin_coords = calculate_pin_coords();
    let (line_cache_x, line_cache_y) = precalculate_all_potential_lines(&pin_coords);
    println!("Drawing lines...");
    let results = calculate_lines(&source_image, &line_cache_y, &line_cache_x);

    let end_time = Instant::now();
    let duration = end_time - start_time;
    println!("Done. Taken {}s.", duration.as_secs_f64());

    println!("\n{:?}", results);
}
fn save_temp_file(buffer: &[f64]) {
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
fn image_to_pixel_array(mut file: File) -> Vec<f64> {
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
        .to_luma_alpha32f()
        .pixels()
        .map(|pixel| {
            let [luma, a] = pixel.0;
            (1.0 - (1.0 - luma) * a) as _
        })
        .collect()
}

fn calculate_pin_coords() -> Vec<Coord> {
    let mut pin_coords = Vec::new();
    let center = IMG_SIZE as f64 / 2.0;
    let radius = (IMG_SIZE / 2) as f64 - 1.0;

    for i in 0..PINS {
        let angle = 2.0 * std::f64::consts::PI * i as f64 / PINS as f64;
        pin_coords.push(Coord {
            x: (center + radius * angle.cos()).floor(),
            y: (center + radius * angle.sin()).floor(),
        });
    }
    pin_coords
}

fn precalculate_all_potential_lines(pin_coords: &Vec<Coord>) -> (Vec<Vec<f64>>, Vec<Vec<f64>>) {
    let mut line_cache_x = vec![vec![]; PINS * PINS];
    let mut line_cache_y = vec![vec![]; PINS * PINS];

    for i in 0..PINS {
        for j in i + MIN_DISTANCE..PINS {
            let x0 = pin_coords[i].x;
            let y0 = pin_coords[i].y;
            let x1 = pin_coords[j].x;
            let y1 = pin_coords[j].y;

            let d = ((x1 - x0).powi(2) + (y1 - y0).powi(2)).sqrt();
            let xs = linspace(x0, x1, d as usize);
            let ys = linspace(y0, y1, d as usize);

            line_cache_x[j * PINS + i] = xs;
            line_cache_y[j * PINS + i] = ys;
        }
    }

    (line_cache_x, line_cache_y)
}

fn linspace(start: f64, end: f64, n: usize) -> Vec<f64> {
    let step = (end - start) / (n as f64 - 1.0);
    (0..n).map(|i| (start + step * i as f64).ceil()).collect()
}

fn calculate_lines(
    source_image: &Vec<f64>,
    line_cache_y: &Vec<Vec<f64>>,
    line_cache_x: &Vec<Vec<f64>>,
) -> Vec<usize> {
    let mut error = source_image
        .iter()
        .copied()
        .map(|pixel| (1.0 - pixel))
        .collect::<Vec<_>>();
    let mut line_sequence = Vec::with_capacity(MAX_LINES);
    let mut last_pins = std::collections::VecDeque::with_capacity(50);

    let mut index = 0;
    let mut current_pin = 0;
    for i in 0..MAX_LINES {
        if i & 511 == 0 {
            save_temp_file(&error);
            println!("{i}/{MAX_LINES}");
        }
        let mut best_pin = 0;
        let mut max_err = std::f64::MIN;

        for offset in MIN_DISTANCE..PINS - MIN_DISTANCE {
            let pin = (current_pin + offset) % PINS;
            if last_pins.contains(&pin) {
                continue;
            } else {
                let inner_index = if pin > current_pin {
                    pin * PINS + current_pin
                } else {
                    current_pin * PINS + pin
                };

                let line_err = get_line_err(
                    &error,
                    &line_cache_y[inner_index],
                    &line_cache_x[inner_index],
                );
                if line_err > max_err {
                    max_err = line_err;
                    best_pin = pin;
                    index = inner_index;
                }
            }
        }

        line_sequence.push(best_pin);

        let coords1 = &line_cache_y[index];
        let coords2 = &line_cache_x[index];
        for i in 0..coords1.len() {
            let v = (coords1[i] * IMG_SIZE as f64) + coords2[i];
            error[v as usize] -= LINE_WEIGHT;
        }

        last_pins.push_back(best_pin);
        if last_pins.len() > 30 {
            last_pins.pop_front();
        }
        current_pin = best_pin;
    }
    let error = source_image
        .iter()
        .copied()
        .zip(error.iter().copied())
        .map(|(s, c)| s + c)
        .collect::<Vec<_>>();
    save_temp_file(&error);
    line_sequence
}

fn get_line_err(err: &Vec<f64>, coords_y: &Vec<f64>, coords_x: &Vec<f64>) -> f64 {
    let mut sum = 0.0;

    for i in 0..coords_y.len() {
        let v = (coords_y[i] * IMG_SIZE as f64) + coords_x[i];
        sum += err[v as usize];
    }
    sum
}
