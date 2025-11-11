use crate::Args;
use crate::canvas::Canvas;
use glam::Vec2;
use rayon::prelude::*;

pub fn calculate_lines(
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
