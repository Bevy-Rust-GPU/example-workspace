use std::path::Path;

use noise_perlin::perlin_2d;

fn main() {
    let width = 256;
    let height = width;

    let mut bytes = vec![0u8; height * width];

    let scale = 2.0;
    let offset = 0.0;

    let constant = 2.0f32.sqrt() / 2.0;
    (bytes.iter_mut().enumerate()).for_each(|(i, pixel)| {
        let x = (i / width) as f32;
        let x = x / width as f32;
        let y = (i % height) as f32;
        let y = y / height as f32;

        let mut v = perlin_2d(x * scale + offset, y * scale + offset);
        v = v * constant + 0.5;
        *pixel = (v * 255.0) as u8;
    });

    image::save_buffer(
        &Path::new("image.png"),
        &bytes,
        width as u32,
        height as u32,
        image::ColorType::L8,
    )
    .unwrap();
}
