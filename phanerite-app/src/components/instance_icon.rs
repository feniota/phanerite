//! Deterministic crystal artwork used to identify Minecraft instances.
//!
//! This is a direct port of `design/src/lib/instance-shapes.ts`'s
//! `crystalFor(seed, { size: 9, maxPrisms: 3, taper: "gradual" })` path.

use std::collections::{HashMap, VecDeque};

use gpui_kit::component::{h_flex, v_flex};
use gpui_kit::{
    App, InteractiveElement as _, IntoElement, ParentElement as _, Styled as _, div,
    prelude::FluentBuilder as _, px,
};

use crate::{
    palette,
    state::{InstanceSummary, Loader},
};

#[derive(Clone)]
struct Prism {
    w: i32,
    h: i32,
    lean: i32,
    shade: i32,
    gap: i32,
    x: i32,
}

fn hash(value: &str) -> u32 {
    // JavaScript iterates UTF-16 code units, not UTF-8 bytes.
    value.encode_utf16().fold(2_166_136_261u32, |hash, unit| {
        (hash ^ unit as u32).wrapping_mul(16_777_619)
    })
}

fn random(seed: &mut u32) -> f32 {
    *seed = seed.wrapping_add(0x6D2B79F5);
    let mut value = *seed;
    value = (value ^ (value >> 15)).wrapping_mul(value | 1);
    value ^= value.wrapping_add((value ^ (value >> 7)).wrapping_mul(value | 61));
    ((value ^ (value >> 14)) as f64 / 4_294_967_296.0) as f32
}

fn rasterize(prisms: &mut [Prism]) -> (HashMap<(i32, i32), usize>, i32, i32, i32, i32) {
    let mut cursor = 0;
    for prism in prisms.iter_mut() {
        prism.x = cursor;
        cursor += (prism.w - 1 + prism.gap).max(1);
    }

    let mut ordered: Vec<Prism> = prisms.to_vec();
    ordered.sort_by_key(|prism| prism.h);
    let mut tones = HashMap::new();

    for prism in &ordered {
        let mut previous_start = None;
        let mut previous_end = None;
        for row in 0..prism.h {
            let from_top = prism.h - 1 - row;
            let tapered = (prism.w - (2 - from_top).max(0)).max(1);
            let inset = (prism.w - tapered) / 2;
            let shift = (prism.lean * row + 2) / 4;
            let mut start = prism.x + inset + shift;
            let mut end = start + tapered - 1;

            if let (Some(previous_start), Some(previous_end)) = (previous_start, previous_end) {
                if start > previous_end {
                    start = previous_end;
                }
                if end < previous_start {
                    end = previous_start;
                }
            }

            let width = end - start + 1;
            for column in 0..width {
                let mut tone = match width {
                    1 => 2,
                    2 if column == 0 => 2,
                    2 => 1,
                    _ if column == 0 => 3,
                    _ if column == width - 1 => 1,
                    _ => 2,
                };
                if from_top <= 1 {
                    tone = (tone + 1).min(3);
                }
                tone = (tone - prism.shade).max(0);
                tones.insert((start + column, -row), tone as usize);
            }
            previous_start = Some(start);
            previous_end = Some(end);
        }
    }

    let min_x = tones.keys().map(|(x, _)| *x).min().unwrap_or(0);
    let max_x = tones.keys().map(|(x, _)| *x).max().unwrap_or(-1);
    let min_y = tones.keys().map(|(_, y)| *y).min().unwrap_or(0);
    let max_y = tones.keys().map(|(_, y)| *y).max().unwrap_or(-1);
    (tones, min_x, min_y, max_x - min_x + 1, max_y - min_y + 1)
}

fn close_holes(cells: &mut [Option<usize>], size: i32) {
    let mut outside = vec![false; cells.len()];
    let mut queue = VecDeque::new();
    for index in 0..cells.len() {
        let x = index as i32 % size;
        let y = index as i32 / size;
        if (x == 0 || y == 0 || x == size - 1 || y == size - 1) && cells[index].is_none() {
            outside[index] = true;
            queue.push_back(index);
        }
    }
    while let Some(index) = queue.pop_front() {
        let x = index as i32 % size;
        let y = index as i32 / size;
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let (nx, ny) = (x + dx, y + dy);
            if nx < 0 || ny < 0 || nx >= size || ny >= size {
                continue;
            }
            let neighbour = (ny * size + nx) as usize;
            if cells[neighbour].is_none() && !outside[neighbour] {
                outside[neighbour] = true;
                queue.push_back(neighbour);
            }
        }
    }
    for index in 0..cells.len() {
        if cells[index].is_some() || outside[index] {
            continue;
        }
        let x = index as i32 % size;
        let y = index as i32 / size;
        let mut tone = 1;
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let (nx, ny) = (x + dx, y + dy);
            if nx >= 0
                && ny >= 0
                && nx < size
                && ny < size
                && let Some(neighbour) = cells[(ny * size + nx) as usize]
            {
                tone = tone.min(neighbour);
            }
        }
        cells[index] = Some(tone);
    }
}

fn crystal(seed: &str, size: i32, max_prisms: i32) -> Vec<Option<usize>> {
    let mut state = hash(seed);
    let count = 2 + (random(&mut state) * (max_prisms - 1).max(1) as f32) as i32;
    let min_height = (size as f32 * 0.34).round().max(3.0) as i32;
    let mut prisms = Vec::new();
    for _ in 0..count {
        let roll = random(&mut state);
        prisms.push(Prism {
            w: if roll < 0.28 {
                1
            } else if roll < 0.78 {
                2
            } else {
                3
            },
            h: min_height + (random(&mut state) * (size - min_height + 1) as f32) as i32,
            lean: (random(&mut state) * 3.0) as i32 - 1,
            shade: if random(&mut state) < 0.45 { 1 } else { 0 },
            gap: if random(&mut state) < 0.65 { 0 } else { 1 },
            x: 0,
        });
    }
    let peak_height = (size as f32 * 0.66).round() as i32;
    let peak = prisms.iter_mut().max_by_key(|prism| prism.h).unwrap();
    peak.h = peak.h.max(peak_height);

    let drawn = loop {
        let drawn = rasterize(&mut prisms);
        if prisms.len() <= 1 || (drawn.3 <= size && drawn.4 <= size) {
            break drawn;
        }
        let shortest = prisms
            .iter()
            .enumerate()
            .min_by_key(|(_, prism)| prism.h)
            .unwrap()
            .0;
        prisms.remove(shortest);
    };

    let (tones, min_x, min_y, width, height) = drawn;
    let mut cells = vec![None; (size * size) as usize];
    let offset_x = (size - width) / 2 - min_x;
    let offset_y = (size - height + 1) / 2 - min_y;
    for ((x, y), tone) in tones {
        let (x, y) = (x + offset_x, y + offset_y);
        if x >= 0 && y >= 0 && x < size && y < size {
            cells[(y * size + x) as usize] = Some(tone);
        }
    }
    close_holes(&mut cells, size);
    cells
}

fn ramp(loader: Loader) -> [u32; 4] {
    match loader {
        Loader::Vanilla => palette::ramp::VANILLA,
        Loader::Fabric => palette::ramp::FABRIC,
        Loader::Forge => palette::ramp::FORGE,
        Loader::NeoForge => palette::ramp::NEOFORGE,
    }
}

pub fn render(instance: &InstanceSummary, cx: &App) -> impl IntoElement {
    render_sized(instance, px(36.), cx)
}

/// Renders the same responsive grid as the Svelte icon. At sizes above 23px,
/// the one-pixel lattice gaps are visible; at 23px and below they collapse.
pub fn render_sized(
    instance: &InstanceSummary,
    size: gpui_kit::Pixels,
    _: &App,
) -> impl IntoElement {
    let cells = crystal(&instance.icon_seed, 9, 3);
    let colors = ramp(instance.loader);
    let show_gaps = size > px(23.);
    let mut rows = v_flex().when(show_gaps, |rows| rows.gap(px(1.)));
    for row in 0..9 {
        let mut cells_row = h_flex().when(show_gaps, |row| row.gap(px(1.)));
        for column in 0..9 {
            let cell = cells[row * 9 + column];
            cells_row = cells_row.child(
                div()
                    .flex_1()
                    .h_full()
                    .when_some(cell, |cell, tone| cell.bg(palette::color(colors[tone]))),
            );
        }
        rows = rows.child(cells_row.flex_1());
    }
    rows.id(format!("instance-icon-{}", instance.id))
        .w(size)
        .h(size)
        .flex_shrink_0()
}
