use std::cmp::max;
use std::path::PathBuf;

use image::io::Reader as ImageReader;
use image::{ImageBuffer, ImageDecoder, Rgb};

// TODO add horizontal concat

/// Loads given images and vertically concatenates them.
/// Images are directly decoded into a single ImageBuffer to avoid unnecessary copying.
///
/// # Arguments
/// * `image_paths` - Vec of PathBufs to images to load
///
/// # Returns
/// * `ImageBuffer<Rgb<u8>, Vec<u8>>`
///
/// # Example
/// ```
/// use image_concat_rs::load_and_vert_concat_images;
/// use std::path::PathBuf;
/// let img_result = load_and_vert_concat_images(&vec![PathBuf::from("./test/1.png"), PathBuf::from("./test/2.png")]);
/// ```
pub fn load_and_vert_concat_images(
    image_paths: &Vec<PathBuf>,
) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>, image::ImageError> {
    // TODO See if threading this would improve speeds
    let mut total_height = 0;
    let mut max_width = 0;

    // Loop through images creating decoders w/o actually reading the images yet
    let mut decoders = Vec::new();
    for path in image_paths {
        let img = ImageReader::open(path)?;
        let decoder = img.into_decoder()?;

        // Track dimensions so we can pre-allocate an ImageBuffer to contain all images
        let (width, height) = decoder.dimensions();
        total_height += height;
        max_width = max(max_width, width);

        decoders.push(decoder);
    }

    // Make an image buffer large enough to contain all images
    let mut buffer: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(max_width, total_height);

    // Loop through decoders, decoding directly into ImageBuffer
    let mut byte_start: u64 = 0;
    for decoder in decoders {
        let byte_len = decoder.total_bytes();
        let byte_end = byte_start + byte_len;

        // Target portion of buffer for n-th image
        let slice = buffer
            .get_mut(byte_start as usize..byte_end as usize)
            .unwrap();

        // Decode image into buffer slice
        let _ = decoder.read_image(slice);

        byte_start = byte_end;
    }

    // Return concatenated images
    Ok(buffer)
}
