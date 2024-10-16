use std::path::PathBuf;

use image::RgbImage;
use image_concat_rs::{
    column_concat_images, concat_images, load_and_column_concat_images,
    load_and_vert_concat_images, ConcatDirection,
};

fn save_img(img: RgbImage, save_path: &str) {
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

    // Load and vertically concat images
    let img = load_and_vert_concat_images(&img_paths)?;
    save_img(img, "./load_and_vert_concat_images.png");

    // Load and concat images into 5 columns
    let img = load_and_column_concat_images(&img_paths, 5)?;
    save_img(img, "./load_and_column_concat_images.png");

    // Load and horizontally concat images using column_concat, likely no performance gain
    let img = load_and_column_concat_images(&img_paths, img_paths.len())?;
    save_img(img, "./horiz_by_load_and_column_concat_images.png");

    // Load images into ImageBuffers
    let imgs: Vec<_> = img_paths
        .iter()
        .map(|path| image::open(path).unwrap().into_rgb8())
        .collect();

    // Concat ImageBuffers Horizontally
    let img = concat_images(&imgs, ConcatDirection::Vertical)?;
    save_img(img, "./concat_images_vert.png");
    // Concat ImageBuffers Vertically
    let img = concat_images(&imgs, ConcatDirection::Horizontal)?;
    save_img(img, "./concat_images_horiz.png");
    // Concat ImageBuffers into 5 columns
    let img = column_concat_images(&imgs, 5)?;
    save_img(img, "./column_concat_images.png");

    // Concat ImageBuffers into 1 too many columns
    let img = column_concat_images(&imgs, imgs.len() + 1)?;
    save_img(img, "./column_concat_images_plus_1.png");

    // Concat ImageBuffers into 2x the columns to ensure blank columns are added
    let img = column_concat_images(&imgs, imgs.len() * 2)?;
    save_img(img, "./column_concat_images_2x.png");

    Ok(())
}
