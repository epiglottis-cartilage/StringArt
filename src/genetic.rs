use crate::Args;
use crate::canvas::Canvas;
use glam::Vec2;
use rand::prelude::*;
use rayon::prelude::*;

#[allow(dead_code)]
pub fn calculate_lines(
    source_image: &Canvas,
    line_cache: &Vec<Vec<Vec<Vec2>>>,
    base: Option<Vec<usize>>,
    args: &Args,
) -> Vec<usize> {
    // 1. 初始化种群
    let mut population = initialize_population(args);
    if let Some(x) = base {
        population.push(x);
    }

    // 2. 遗传算法主循环
    for i in 0..args.generations {
        eprintln!("Generation {}/{}", i, args.generations);

        // 计算种群适应度
        let fitness_scores: Vec<f32> = population
            .par_iter()
            .map(|chromosome| calculate_fitness(chromosome, source_image, line_cache, args))
            .collect();

        // 选择下一代父代
        let mut new_population = Vec::with_capacity(args.population_size);
        for _ in 0..args.population_size {
            let parent1 = tournament_selection(&population, &fitness_scores, 3);
            let parent2 = tournament_selection(&population, &fitness_scores, 3);

            // 交叉
            let child = if rand::random::<f32>() < args.crossover_rate {
                crossover(parent1, parent2)
            } else {
                parent1.clone()
            };

            // 变异
            let child = if rand::random::<f32>() < args.mutation_rate {
                mutate(&child, args)
            } else {
                child
            };

            new_population.push(child);
        }

        population = new_population;
    }

    // 3. 返回最优个体
    let best_idx = population
        .iter()
        .map(|c| calculate_fitness(c, source_image, line_cache, args))
        .enumerate()
        .max_by(|(_, f1), (_, f2)| f1.total_cmp(f2))
        .map(|(i, _)| i)
        .unwrap();
    population.remove(best_idx)
}

fn initialize_population(args: &Args) -> Vec<Vec<usize>> {
    let mut population = Vec::with_capacity(args.population_size);
    for _ in 0..args.population_size {
        let mut chromosome = vec![0; args.lines + 1]; // 线条数+1个引脚（首尾衔接）
        let mut current_pin = 0;
        for i in 1..=args.lines {
            // 随机选择符合间距限制的下一个引脚（同原逻辑）
            let offset = (args.distance..args.pin - args.distance)
                .choose(&mut rand::thread_rng())
                .unwrap();
            current_pin = (current_pin + offset) % args.pin;
            chromosome[i] = current_pin;
        }
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
    // 计算线条序列的误差（与目标图像的差异）
    let mut error = source_image.clone();
    error.invert(); // 同原逻辑：误差初始为目标图像的反相

    for window in chromosome.windows(2) {
        let (s, e) = (window[0], window[1]);
        let (s, e) = if s < e { (s, e) } else { (e, s) };
        for point in &line_cache[s][e] {
            *error.get_pixel_mut(*point) -= args.line_weight;
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

    // 复制父代1的[start, end)片段，其余复制父代2
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
    let pos = rand::random::<usize>() % (chromosome.len() - 1); // 避免最后一个位置（无后续）
    let current = chromosome[pos];
    // 随机选择符合间距限制的新引脚
    let offset = (args.distance..args.pin - args.distance)
        .choose(&mut rand::thread_rng())
        .unwrap();
    mutated[pos + 1] = (current + offset) % args.pin;
    mutated
}
