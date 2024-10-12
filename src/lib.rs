use std::cmp::max;
use std::path::PathBuf;

use image::io::Reader as ImageReader;
use image::{GenericImage, ImageBuffer, ImageDecoder, Pixel, RgbImage};

/// Loads given images and vertically concatenates them.
/// Images are directly decoded into a single ImageBuffer to avoid unnecessary copying.
///
/// # Arguments
/// * `image_paths` - Slice of PathBufs to images to load
///
/// # Returns
/// * `RgbImage`
///
/// # Example
/// ```
/// use image_concat_rs::load_and_vert_concat_images;
/// use std::path::PathBuf;
/// let img_result = load_and_vert_concat_images(&vec![PathBuf::from("./test/1.png"), PathBuf::from("./test/2.png")]);
/// // or
/// let img_result = load_and_vert_concat_images(&[PathBuf::from("./test/1.png"), PathBuf::from("./test/2.png")]);
/// ```
pub fn load_and_vert_concat_images(image_paths: &[PathBuf]) -> Result<RgbImage, image::ImageError> {
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
    let mut buffer: RgbImage = ImageBuffer::new(max_width, total_height);

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
/// * `RgbImage`
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
) -> Result<RgbImage, image::ImageError> {
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

    concat_images(&col_buffs, ConcatDirection::Horizontal)
}

pub enum ConcatDirection {
    Vertical,
    Horizontal,
}

/// Concatenates ImageBuffers vertically or horizontally
///
/// # Arguments
/// * `images` - Slice of ImageBuffers to concatenate
/// * `direction` - ConcatDirection::Vertical or ConcatDirection::Horizontal
///
/// # Returns
/// * `Result<ImageBuffer, image::ImageError>`
///
/// # Example
/// ```
/// use image_concat_rs::{concat_images, ConcatDirection};
/// let img1 = image::open("./test/1.png").unwrap().into_rgb8();
/// let img2 = image::open("./test/2.png").unwrap().into_rgb8();
/// let img_result = concat_images(&[img1,img2], ConcatDirection::Vertical);
/// ```
pub fn concat_images<P: Pixel>(
    images: &[ImageBuffer<P, Vec<P::Subpixel>>],
    direction: ConcatDirection,
) -> Result<ImageBuffer<P, Vec<P::Subpixel>>, image::ImageError> {
    let blits = get_concat_blits(images, direction, 0, 0);
    place_images_in_buffer(&blits)
}

pub struct ImageBlit<'a, P: Pixel> {
    pub img: &'a ImageBuffer<P, Vec<P::Subpixel>>,
    pub x: u32,
    pub y: u32,
    // TODO could probably add origin pretty easily.
    // - One complication that comes to mind is a non top left origin on left or
    //   top boundary would cause the image buffer to grow to accomodate which
    //   would then offset all other image placements. Would need to add logic
    //   to clip images probably.
}

/// Places ImageBuffers into a single buffer
///   
/// The list of images and placements will be scanned to determine the total size
/// of the buffer then all images will be copied into the buffer.
///
/// The goal of this function is to enable direction agnostic concatenation with
/// as few copies as possible. For example, instead of doing column concatenation
/// by creating a column of images and then horizontally concatenating them,
/// which would require an unnecessary copy of the columns into the final
/// horizontal alignment, this takes all the desired placements and copies them
/// into a single buffer.
///
/// # Arguments
/// * `images` - Slice of ImageBlit structs which contain an ImageBuffer ref and
///  target coordinate to place the top left of the image
///
/// # Returns
/// * `ImageBuffer` - Single ImageBuffer containing all images
///
/// # Example
/// ```
/// use image_concat_rs::{place_images_in_buffer,ImageBlit};
/// let img1 = image::open("./test/1.png").unwrap().into_rgb8();
/// let img2 = image::open("./test/2.png").unwrap().into_rgb8();
/// let img_result = place_images_in_buffer(&[ImageBlit{img: &img1, x: 0, y: 0}, ImageBlit{img: &img2, x: img1.width(), y: 0}]);
/// ```
pub fn place_images_in_buffer<P: Pixel>(
    images: &[ImageBlit<P>],
) -> Result<ImageBuffer<P, Vec<P::Subpixel>>, image::ImageError> {
    // Each each images start point and dimensions to determine the total buffer size we'll need to contain everything
    let (total_width, total_height) =
        images.iter().fold((0, 0), |(max_width, max_height), blit| {
            (
                max(max_width, blit.x + blit.img.width()),
                max(max_height, blit.y + blit.img.height()),
            )
        });

    // Create an image buffer large enough to contain all images
    let mut buffer = ImageBuffer::new(total_width, total_height);

    // Copy each image into the final buffer
    for blit in images {
        buffer.copy_from(blit.img, blit.x, blit.y)?;
    }

    Ok(buffer)
}

/// Creates a Vector of ImageBlit structs
///
/// Takes start location and concat direction to create blits that will vertically or horizontally cocnatenate ImageBuffers
///
/// # Arguments
/// * `images` - Slice of ImageBuffers to concatenate
/// * `concat_direction` - ConcatDirection::Vertical or ConcatDirection::Horizontal
/// * `start_y` - y coord that the origin of the first image will be placed
/// * `start_x` - x coord that the origin of the first image will be placed
///
/// # Returns
/// * Vec of ImageBlit structs that can be passed to place_images_in_buffer to draw all images to a single buffer
///
/// # Example
/// ```
/// use image_concat_rs::{get_concat_blits, ConcatDirection};
/// let img1 = image::open("./test/1.png").unwrap().into_rgb8();
/// let img2 = image::open("./test/2.png").unwrap().into_rgb8();
/// let blits = get_concat_blits(&[img1,img2], ConcatDirection::Vertical, 0, 0);
/// ```
pub fn get_concat_blits<P: Pixel>(
    images: &[ImageBuffer<P, Vec<P::Subpixel>>],
    concat_direction: ConcatDirection,
    start_x: u32,
    start_y: u32,
) -> Vec<ImageBlit<P>> {
    // Strep through each image and create an ImageBlit with start relative to the previous image's width or height depending on the concat direction
    let (blits, _) = images.iter().fold(
        (Vec::new(), (start_x, start_y)),
        |(mut blits, (x, y)), img| {
            let blit = ImageBlit { img, x, y };
            blits.push(blit);
            match concat_direction {
                ConcatDirection::Vertical => (blits, (x, y + img.height())),
                ConcatDirection::Horizontal => (blits, (x + img.width(), y)),
            }
        },
    );

    blits
}

/// Concatenates images into columns
///
/// This will take already loaded images and concatenate them in vertical columns.
///
/// Given a desired number of columns, it will divde them as evenly as possible,
/// placing what will evenly divide into all columns and spreading the remainders
/// across the front columns.
///
/// The order is currently top to bottom, moving to the next column from left to right.
/// This order might change as it makes knowing where empty rows are a bit unintuitive.
///
/// # Arguments
/// * `images` - Slice of ImageBuffers to concatenate in columns
/// * `columns` - Number of columns to split images into
///
/// # Returns
/// * `Result<ImageBuffer, image::ImageError>`
///
/// # Example
/// ```
/// use image_concat_rs::{column_concat_images, ConcatDirection};
/// let img1 = image::open("./test/1.png").unwrap().into_rgb8();
/// let img2 = image::open("./test/2.png").unwrap().into_rgb8();
/// let img_result = column_concat_images(&[img1,img2], 2);
///
/// ```
pub fn column_concat_images<P: Pixel>(
    images: &[ImageBuffer<P, Vec<P::Subpixel>>],
    columns: usize,
) -> Result<ImageBuffer<P, Vec<P::Subpixel>>, image::ImageError> {
    let num_images = images.len();

    // Max number of images per column
    let chunk_size = num_images / columns;
    // Starting index of columns that will have less images
    let chunk_remainder = num_images % columns;

    // vec of ImageBlit instructions we will execute all at once after planning the columns
    let mut blits = Vec::with_capacity(num_images);

    // Build column image blits
    let mut start = 0;
    let mut x = 0;
    for idx in 0..columns {
        // Determine if this is a full size column or a partial column
        let chunk_size = if idx < chunk_remainder {
            chunk_size + 1
        } else {
            chunk_size
        };
        let end = start + chunk_size;

        // Exit early if there are no images in this column
        if end >= num_images {
            break;
        }

        // create a list of ImageBlits to draw a column of images
        let col_blits = get_concat_blits(&images[start..end], ConcatDirection::Vertical, x, 0);

        // determine x coord of next column by finding the widest blit
        let max_width = col_blits
            .iter()
            .map(|blit| blit.x + blit.img.width())
            .max()
            .unwrap();
        // account for current x coord so only current image width is considered
        let max_width = max_width - x;

        // add blits to blit buffer
        blits.extend(col_blits);

        // set next column starting x coord
        x += max_width;

        // update image index
        start = end;
    }

    // execute all blits
    place_images_in_buffer(&blits)
}

mod tests {
    use crate::load_and_column_concat_images;

    #[test]
    fn test_concat_images() {
        let imgs = vec![
            image::open("./test/1.png").unwrap().into_rgb8(),
            image::open("./test/2.png").unwrap().into_rgb8(),
        ];
        let expected_w = imgs.iter().map(|img| img.width()).max().unwrap();
        let expected_h: u32 = imgs.iter().map(|img| img.height()).sum();

        let img_result = super::concat_images(&imgs, super::ConcatDirection::Vertical).unwrap();
        // TODO maybe check against gold images
        assert_eq!(img_result.width(), expected_w);
        assert_eq!(img_result.height(), expected_h);
    }

    #[test]
    fn test_column_concat_images_unbalanced() {
        let single_img = vec![image::open("./test/1.png").unwrap().into_rgb8()];
        // request concatting 2 columns, but only pass 1 image
        let _img_result = super::column_concat_images(&single_img, 2).unwrap();
    }
}
