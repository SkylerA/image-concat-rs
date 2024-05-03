use std::cmp::max;
use std::path::PathBuf;

use image::io::Reader as ImageReader;
use image::{GenericImage, ImageBuffer, ImageDecoder, Rgb};

// TODO Make Image type more generic

// NOTE: This currently assumes images are the same dimensions

/// Loads given images and vertically concatenates them.
/// Images are directly decoded into a single ImageBuffer to avoid unnecessary copying.
///
/// # Arguments
/// * `image_paths` - Slice of PathBufs to images to load
///
/// # Returns
/// * `ImageBuffer<Rgb<u8>, Vec<u8>>`
///
/// # Example
/// ```
/// use image_concat_rs::load_and_vert_concat_images;
/// use std::path::PathBuf;
/// let img_result = load_and_vert_concat_images(&vec![PathBuf::from("./test/1.png"), PathBuf::from("./test/2.png")]);
/// // or
/// let img_result = load_and_vert_concat_images(&[PathBuf::from("./test/1.png"), PathBuf::from("./test/2.png")]);
/// ```
pub fn load_and_vert_concat_images(
    image_paths: &[PathBuf],
) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>, image::ImageError> {
    let mut total_height = 0;
    let mut max_width = 0;

    // Loop through images creating decoders w/o actually reading the images yet
    let mut decoders = Vec::new();
    for path in image_paths {
        let img = ImageReader::open(path).map_err(|err| {
            std::io::Error::new(
                err.kind(),
                format!("Error opening image {}: {}", path.to_str().unwrap(), err),
            )
        })?;

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

/// Loads given images and concatenate them into columns.
/// Images are directly decoded into vertical columns to avoid unnecessary copying,
/// but horizontal concatenation of those columns requires copying of already decoded images.
///
/// # Arguments
/// * `image_paths` - Slice of PathBufs to images to load
/// * `columns` - number of columns to split images into
///
/// # Returns
/// * `ImageBuffer<Rgb<u8>, Vec<u8>>`
///
/// # Example
/// ```
/// use image_concat_rs::load_and_column_concat_images;
/// use std::path::PathBuf;
/// let img_result = load_and_column_concat_images(&[PathBuf::from("./test/1.png"), PathBuf::from("./test/2.png")], 2);
/// ```
pub fn load_and_column_concat_images(
    image_paths: &[PathBuf],
    columns: usize,
) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>, image::ImageError> {
    // Vertical concatenation is more performant than horizontal because we can use the contiguous
    // nature of the memory to directly decode images into a final buffer one after another without
    // making copies of data. Horitontal concatenation would require decoding one row of each image
    // into a final buffer before moving to the next line which I don't see a way to do in ImageDecoder.
    // As such, we'll performantly vertical concatenate columns of images and then horizontally
    // concatenate the columns into a single image buffer.
    // Unfortunately, the horizontal concatenation will require explicitly copying memory over.

    // vec to store our vertically concatenated columns
    let mut col_buffs = Vec::new();

    // Max number of images per column
    let chunk_size = image_paths.len() / columns;
    // Starting index of columns that will have less images
    let chunk_remainder = image_paths.len() % columns;

    // Build image columns
    let mut start = 0;
    for idx in 0..columns {
        // Determine if this is a full size column or a partial column
        let chunk_size = if idx < chunk_remainder {
            chunk_size + 1
        } else {
            chunk_size
        };
        let end = start + chunk_size;

        // Grab dynamic chunk size of images and concat verically
        let buff = load_and_vert_concat_images(&image_paths[start..end])?;
        col_buffs.push(buff);

        start = end;
    }

    Ok(concat_images(&col_buffs, ConcatDirection::Horizontal)?)
}

pub enum ConcatDirection {
    Vertical,
    Horizontal,
}
/// Concatenates images vertically or horizontally
///
/// # Arguments
/// * `images` - Slice of PathBufs to images to load
/// * `direction` - ConcatDirection::Vertical or ConcatDirection::Horizontal
///
/// # Returns
/// * `Result<ImageBuffer<Rgb<u8>, Vec<u8>, image::ImageError>`
///
/// # Example
/// ```
/// use image_concat_rs::{concat_images, ConcatDirection};
/// let img1 = image::open("./test/1.png").unwrap().into_rgb8();
/// let img2 = image::open("./test/2.png").unwrap().into_rgb8();
/// let img_result = concat_images(&[img1,img2], ConcatDirection::Vertical);
/// ```
pub fn concat_images(
    images: &[ImageBuffer<Rgb<u8>, Vec<u8>>],
    direction: ConcatDirection,
) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>, image::ImageError> {
    match direction {
        ConcatDirection::Vertical => {
            // Find the max width and total height of all images
            let (max_width, total_height) =
                images
                    .iter()
                    .fold((0, 0), |(max_width, total_height), img| {
                        (max(max_width, img.width()), total_height + img.height())
                    });

            let mut buffer = ImageBuffer::new(max_width, total_height);

            // Copy each image into the final buffer
            let mut write_start = 0;
            for img in images {
                let write_end = write_start + img.height();
                buffer.copy_from(img, 0, write_start)?;
                write_start = write_end;
            }

            Ok(buffer)
        }
        ConcatDirection::Horizontal => {
            // Find the total width and max height of all images
            let (total_width, max_height) =
                images
                    .iter()
                    .fold((0, 0), |(total_width, max_height), img| {
                        (total_width + img.width(), max(max_height, img.height()))
                    });

            let mut buffer = ImageBuffer::new(total_width, max_height);

            // Copy each image into the final buffer
            let mut write_start = 0;
            for img in images {
                let write_end = write_start + img.width();
                buffer.copy_from(img, write_start, 0)?;
                write_start = write_end;
            }

            Ok(buffer)
        }
    }
}
