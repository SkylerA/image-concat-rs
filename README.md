# image-concat-rs

Helper function to vertically concatenate images by directly decoding them into an ImageBuffer to avoid unnecessary memory copying.

## Example

```rust
use image_concat_rs::load_and_vert_concat_images;
use std::path::PathBuf;
let img_result = load_and_vert_concat_images(&vec![PathBuf::from("./test/1.png"), PathBuf::from("./test/2.png")]);
```
