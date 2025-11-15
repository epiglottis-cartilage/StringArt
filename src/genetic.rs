use crate::canvas::Canvas;
use crate::{Args, tabu};
use glam::Vec2;
use rand::prelude::*;
use rayon::prelude::*;

#[allow(dead_code)]
pub fn calculate_lines(
    source_image: &Canvas,
    line_cache: &Vec<Vec<Vec<Vec2>>>,
    args: &Args,
) -> Vec<usize> {
    let mut population = initialize_population(source_image, line_cache, args);

    for i in 0..args.generations {
        eprintln!("Generation {}/{}", i, args.generations);

        let fitness_scores: Vec<f32> = population
            .par_iter()
            .map(|chromosome| calculate_fitness(chromosome, source_image, line_cache, args))
            .collect();

        let mut new_population = Vec::with_capacity(args.population_size);
        for _ in 0..args.population_size {
            let parent1 = tournament_selection(&population, &fitness_scores, 3);
            let parent2 = tournament_selection(&population, &fitness_scores, 3);

            let child = if rand::random::<f32>() < args.crossover_rate {
                crossover(parent1, parent2)
            } else {
                parent1.clone()
            };

            let child = if rand::random::<f32>() < args.mutation_rate {
                mutate(&child, args)
            } else {
                child
            };

            new_population.push(child);
        }

        population = new_population;
    }

    let best_idx = population
        .iter()
        .map(|c| calculate_fitness(c, source_image, line_cache, args))
        .enumerate()
        .max_by(|(_, f1), (_, f2)| f1.total_cmp(f2))
        .map(|(i, _)| i)
        .unwrap();
    population.remove(best_idx)
}

fn initialize_population(
    source_image: &Canvas,
    line_cache: &Vec<Vec<Vec<Vec2>>>,
    args: &Args,
) -> Vec<Vec<usize>> {
    let mut population = Vec::with_capacity(args.population_size);
    for i in 0..args.population_size {
        eprintln!("Initializing chromosome {}/{}", i, args.population_size);
        let chromosome =
            tabu::calculate_lines(source_image, line_cache, &tabu::Config::rand_from(args));
        population.push(chromosome);
    }
    population
}
fn calculate_fitness(
    chromosome: &[usize],
    source_image: &Canvas,
    line_cache: &[Vec<Vec<Vec2>>],
    args: &Args,
) -> f32 {
    let mut error = source_image.clone();
    error.invert();

    for window in chromosome.windows(2) {
        let (s, e) = (window[0], window[1]);
        let (s, e) = if s < e { (s, e) } else { (e, s) };
        for point in &line_cache[s][e] {
            *error.get_pixel_mut(*point) =
                (*error.get_pixel_mut(*point) - args.line_weight).max(0.);
        }
    }

    let total_error: f32 = error.buf.pixels().map(|p| p.0[0].abs()).sum();
    -total_error
}
fn tournament_selection<'a>(
    population: &'a [Vec<usize>],
    fitness: &[f32],
    k: usize,
) -> &'a Vec<usize> {
    let mut best_idx = rand::random::<usize>() % population.len();
    for _ in 1..k {
        let idx = rand::random::<usize>() % population.len();
        if fitness[idx] > fitness[best_idx] {
            best_idx = idx;
        }
    }
    &population[best_idx]
}
fn crossover(parent1: &[usize], parent2: &[usize]) -> Vec<usize> {
    let len = parent1.len();
    let mut child = vec![0; len];
    let (p1, p2) = (rand::random::<usize>() % len, rand::random::<usize>() % len);
    let (start, end) = (p1.min(p2), p1.max(p2));

    for i in 0..len {
        child[i] = if i >= start && i < end {
            parent1[i]
        } else {
            parent2[i]
        };
    }
    child
}
fn mutate(chromosome: &[usize], args: &Args) -> Vec<usize> {
    let mut mutated = chromosome.to_vec();
    let pos = rand::random::<usize>() % (chromosome.len() - 1);
    // 随机选择符合间距限制的新引脚
    // let current = chromosome[pos];
    // let offset = (args.distance..args.pin - args.distance)
    //     .choose(&mut rand::thread_rng())
    //     .unwrap();
    // mutated[pos + 1] = (current + offset) % args.pin;

    mutated[pos + 1] = random::<usize>() % args.pin;
    mutated
}
