use std::path::PathBuf;

use image::{ImageBuffer, Rgb};
use image_concat_rs::{concat_images, load_and_column_concat_images, load_and_vert_concat_images};

fn save_img(img: ImageBuffer<Rgb<u8>, Vec<u8>>, save_path: &str) {
    match img.save_with_format(save_path, image::ImageFormat::Png) {
        Ok(_) => println!("Saved image to {save_path}"),
        Err(err) => println!("Error saving to {save_path}: {err}"),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Make a Vec of image PathBufs
    let img_count = 8;
    let img_paths: Vec<_> = (1..=img_count)
        .map(|i| format!("./test/{}.png", i))
        .map(|s| PathBuf::from(&s))
        .collect();

    // Load and concat images
    let img = load_and_vert_concat_images(&img_paths)?;
    save_img(img, "./load_and_vert_concat_images.png");

    let img = load_and_column_concat_images(&img_paths, 5)?;
    save_img(img, "./load_and_column_concat_images.png");

    let imgs: Vec<_> = img_paths
        .iter()
        .map(|path| image::open(path).unwrap().into_rgb8())
        .collect();

    let img = concat_images(&imgs, image_concat_rs::ConcatDirection::Vertical)?;
    save_img(img, "./concat_images_vert.png");
    let img = concat_images(&imgs, image_concat_rs::ConcatDirection::Horizontal)?;
    save_img(img, "./concat_images_horiz.png");

    Ok(())
}
