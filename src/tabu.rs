use crate::Args;
use crate::canvas::Canvas;
use glam::Vec2;
use rayon::prelude::*;
#[derive(Debug, Clone)]
pub struct Config {
    /// number of pins to draw.
    /// number for no reason
    pin: usize,
    /// how many lines should calculate.
    lines: usize,
    /// weight of the lines.
    pub line_weight: f32,
    /// size of the image.
    /// bigger means more accurate but slower.
    pub _img_size: u32,
    /// distance between two pins we choose.
    pub distance: usize,
    /// how long should we omit a pin after it is used.
    pub tabu: usize,
    pub start: usize,
}
impl Config {
    pub fn rand_from(args: &Args) -> Self {
        fn random_in(val: f64, percentage: f64) -> f64 {
            let frac = rand::random::<f64>() * (percentage * 2.) + (1. - percentage);
            val as f64 * frac
        }
        Self {
            pin: args.pin,
            lines: args.lines,
            line_weight: random_in(args.line_weight as _, 0.25) as _,
            _img_size: random_in(args.img_size as _, 0.25) as _,
            distance: random_in(args.distance as _, 0.5) as _,
            tabu: random_in(args.tabu as _, 0.5) as _,
            start: rand::random::<u32>() as usize % args.pin,
        }
    }
}

#[allow(dead_code)]
pub fn calculate_lines(
    source_image: &Canvas,
    line_cache: &Vec<Vec<Vec<Vec2>>>,
    args: &Config,
) -> Vec<usize> {
    let mut error = source_image.clone();
    error.invert();
    let mut line_sequence = Vec::with_capacity(args.lines);
    let mut recent_pins = std::collections::VecDeque::with_capacity(args.tabu + 1);

    let mut current_pin = args.start;
    line_sequence.push(args.start);

    for _i in 0..args.lines {
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
            *error.get_pixel_mut(*point) =
                (*error.get_pixel_mut(*point) - args.line_weight).max(0.);
        })
    }

    line_sequence
}
