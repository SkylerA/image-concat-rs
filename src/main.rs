use std::path::PathBuf;

use image_concat_rs::load_and_vert_concat_images;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Make a Vec of image PathBufs
    let img_count = 2;
    let img_paths: Vec<_> = (1..=img_count)
        .map(|i| format!("./test/{}.png", i))
        .map(|s| PathBuf::from(&s))
        .collect();

    // Load and concat images
    let img = load_and_vert_concat_images(&img_paths)?;

    // Save image
    let save_path = "./out.png";
    match img.save_with_format(save_path, image::ImageFormat::Png) {
        Ok(_) => println!("Saved image to {save_path}"),
        Err(err) => println!("Error saving to {save_path}: {err}"),
    }

    Ok(())
}
