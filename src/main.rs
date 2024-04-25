use std::cmp::max;
use std::ops::Div;

use image::imageops::overlay;
use image::io::Reader as ImageReader;
use image::{ImageBuffer, ImageDecoder, Rgb};

fn load_and_concat_images_buffer(image_paths: &Vec<&str>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    // TODO consider using actual Paths
    // TODO See if threading this would improve speeds
    let mut total_height = 0;
    let mut max_width = 0;

    let mut decoders = Vec::new();

    for path in image_paths {
        let img = ImageReader::open(path).unwrap();
        let decoder = img.into_decoder().unwrap();
        let (width, height) = decoder.dimensions();
        total_height += height;
        max_width = max(max_width, width);
        // println!("Loading {} ({}x{})", path, width, height);

        decoders.push(decoder);
    }

    // Make an image buffer large enough to contain all images
    let mut buffer: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(max_width, total_height);

    let mut byte_start: u64 = 0;
    // Loop through decoders, decoding directly into ImageBuffer
    for decoder in decoders {
        let byte_len = decoder.total_bytes();
        let byte_end = byte_start + byte_len;
        // println!("Writing {} bytes", byte_len);

        // Target portion of buffer for n-th image
        let slice = buffer
            .get_mut(byte_start as usize..byte_end as usize)
            .unwrap();

        // Decode image into buffer slice
        let _ = decoder.read_image(slice);

        byte_start = byte_end;
    }

    buffer
}

fn load_and_concat_images_copies(image_paths: &Vec<&str>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let mut total_height = 0;
    let mut max_width = 0;

    let mut images = Vec::new();

    for path in image_paths {
        // A more direct comparison would be waiting to read the data until we are in the
        // concat loop but we need image dimensions to pre-allocate an ImageBuffer
        // which requires to_decoder() or decode() (I think) and once you've called
        // to_decoder() you will eventually read directly into a buffer so it's rather
        // pointless to read it into a buffer and then draw it in with overlay instead
        // of just writing directly to the ImageBuffer
        // Alternatively you could also try to grow the ImageBuffer with each new img
        let img = ImageReader::open(path)
            .unwrap()
            .decode()
            .unwrap()
            .into_rgb8();
        let (width, height) = img.dimensions();
        total_height += height;
        max_width = max(max_width, width);
        // println!("Loading {} ({}x{})", path, width, height);

        // Push already decoded image into vec, this most likely causes a copy of memory.
        images.push(img);
    }

    let mut buffer: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(max_width, total_height);

    let mut y_offset = 0;
    for img in images {
        // Draw image onto specific portion of ImageBuffer
        overlay(&mut buffer, &img, 0, y_offset);
        y_offset += img.height() as i64;
    }

    buffer
}

fn main() {
    use std::time::Instant;
    let img_count = 60;
    let img_paths: Vec<_> = (1..img_count)
        .map(|i| format!("./test_imgs/nfl_clock/clock/{}.png", i))
        .collect();

    let img_paths_str: Vec<&str> = img_paths.iter().map(AsRef::as_ref).collect();

    let test_fns: Vec<(&str, fn(&Vec<&str>) -> ImageBuffer<Rgb<u8>, Vec<u8>>)> = vec![
        ("Buffer", load_and_concat_images_buffer),
        ("Copies", load_and_concat_images_copies),
    ];

    for (name, f) in test_fns {
        let now = Instant::now();

        let loop_count = 100;
        for _ in 0..loop_count {
            f(&img_paths_str);
        }
        let elapsed = now.elapsed();
        println!(
            "{name} - Time to concat {img_count} images {loop_count} times: {:.2?} avg: {:.2?}",
            elapsed,
            elapsed.div(loop_count)
        );
    }
}
