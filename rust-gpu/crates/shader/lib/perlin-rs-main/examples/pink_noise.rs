use std::path::Path;

use noise_perlin::perlin_2d;

static SQRT_2_OVER_2: f32 = 0.70710678118;

fn apply_noise(
    width: usize,
    height: usize,
    frequency: f32,
    amplitude: f32,
    offset: f32,
    values: &mut Vec<f32>,
) {
    (values.iter_mut().enumerate()).for_each(|(i, entry)| {
        let x = (i / width) as f32;
        let x = x / width as f32;
        let y = (i % height) as f32;
        let y = y / height as f32;

        let mut v = perlin_2d(x * frequency + offset, y * frequency + offset);
        v = v * SQRT_2_OVER_2 + 0.5;
        *entry += v * amplitude;
    });
}

fn main() {
    let width = 512;
    let height = width;

    // alpha of the pink noise, the scaling of amplitude with frequency
    let alpha: f32 = 1.0;

    let mut values = vec![0.0f32; height * width];

    for scale in 0..8u32 {
        let frequency = 2.0f32.powf(scale as f32);
        let amplitude = 1.0 / frequency.powf(alpha);
        apply_noise(width, height, frequency, amplitude, 0.0, &mut values)
    }

    let max = values.iter().cloned().fold(0. / 0., f32::max);
    let min = values.iter().cloned().fold(0. / 0., f32::min);
    let bytes = values
        .into_iter()
        .map(|x| ((x - min) / (max - min) * 255.0) as u8)
        .collect::<Vec<_>>();

    image::save_buffer(
        &Path::new("pink.png"),
        &bytes,
        width as u32,
        height as u32,
        image::ColorType::L8,
    )
    .unwrap();
}
