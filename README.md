# image-concat-rs

Doing some testing to see if directly decoding images into an ImageBuffer has a notable speed gain over a more simple approach. Might not be a 1 to 1 comparison as my concat methods are different in each due to my limited experience with the Image types.

## Result

In a tight loop with lots of images, writing to buffer will be beneficial

> Buffer - Time to concat 60 images 100 times: 8.22s avg: 82.23ms  
> Copies - Time to concat 60 images 100 times: 14.47s avg: 144.75ms

Copying method has an extra copy when pushing already decoded images to a vector, some speed up might be gained by adding them directly to the ImageBuffer in the loop altho you'd have to extend the buffer as you go instead of pre-allocating it all after looping through the images and determining sizes. In the end, writing directly to an ImageBuffer is easy enough so not worth the further comparison.
