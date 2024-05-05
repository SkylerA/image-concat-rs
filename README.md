# image-concat-rs

This contains a few performance focused image concatenation functions to help reduce needless copies. The goal is to make image concatenation a bit more efficient when used in a tight loop like processing video frames etc.

## Loading Images and Concatenating

`load_and_vert_concat_images` attempts to improve loading from disk by avoiding any extra copying. It opens images as `ImageDecoder`s and then directly decodes them into a pre-sized ImageBuffer.

`load_and_column_concat_images` partially achieves direct decoding by using `load_and_vert_concat_images` for each column, but since `ImageDecoder` decodes directly to a buffer of contiguous memory, horizontal concatenation can't be directly decoded because you'd need to read 1 row of each image into the final buffer and then move to the next row.

No horizontal concate function is provided as there is no performance gain and it can be achieved with `load_and_column_concat_images` by setting the `columns` parameter to `image_paths.len()`.

## Concatenating ImageBuffers

If you are working with already loaded images and need to perform a series of concatenations, slight performance gains can be achieve by creating a list of image placements with the `ImageBlit` struct which specifies an image and `x`,`y` coodinates to place the image. A slice of `ImageBlit`s can be passed to `place_images_in_buffer` which will determine the necessary `ImageBuffer` size execute the placement of the images with `ImageBuffer::copy_from`.

`concat_images` is provided for horizontally or vertically concatenating ImageBuffers  
`column_concat_images` will split a slice of `ImageBuffer`s into columns and place them all in one final ImageBuffer instead of concatenating into columns and then concatenating those columns horizontally which should reduce some memory copies.

`get_concat_blits` can be used to create a vector of `ImageBlit`s with horizontal or vertical concetnation starting from a specific point. A collection of these vectors can be combined and passed to `place_images_in_buffer` to execute a series of image placements into 1 final `ImageBuffer` without performing needless copies that a complex series of concatenations might have required.

## Example

```rust
use std::path::PathBuf;
use image_concat_rs::{
    column_concat_images, concat_images, load_and_column_concat_images, load_and_vert_concat_images, ConcatDirection,
};

// Make a Vec of image PathBufs
let img_count = 8;
let img_paths: Vec<_> = (1..=img_count)
    .map(|i| format!("./test/{}.png", i))
    .map(|s| PathBuf::from(&s))
    .collect();

// Load and vertically concat images
let img = load_and_vert_concat_images(&img_paths)?;
// Load and concat images into 5 columns
let img = load_and_column_concat_images(&img_paths, 5)?;
// Load and horizontally concat images using column_concat, likely no performance gain
let img = load_and_column_concat_images(&img_paths, img_paths.len())?;

// Load images into ImageBuffers
let imgs: Vec<_> = img_paths
    .iter()
    .map(|path| image::open(path).unwrap().into_rgb8())
    .collect();

// Concat ImageBuffers Horizontally
let img = concat_images(&imgs, ConcatDirection::Vertical)?;
// Concat ImageBuffers Vertically
let img = concat_images(&imgs, ConcatDirection::Horizontal)?;
// Concat ImageBuffers into 5 columns
let img = column_concat_images(&imgs, 5)?;
```
