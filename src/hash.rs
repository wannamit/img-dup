extern crate image;

use dct::{dct_2d, crop_dct};

use image::{GenericImage, DynamicImage, 
    ImageBuf, Luma, Pixel, FilterType, Nearest};
use image::imageops::{grayscale, resize};

use std::collections::Bitv;

static FILTER_TYPE: FilterType = Nearest;

#[deriving(PartialEq, Eq, Hash, Show, Clone)]
pub struct ImageHash {
    size: u32,
    bitv: Bitv,
}

impl ImageHash {

    pub fn dist(&self, other: &ImageHash) -> uint {
        assert!(self.bitv.len() == other.bitv.len(), 
                "ImageHashes must be the same length for proper comparison!");

        self.bitv.iter().zip(other.bitv.iter())
            .filter(|&(left, right)| left != right).count()
    }

    pub fn dist_ratio(&self, other: &ImageHash) -> f32 {
        self.dist(other) as f32 / self.size as f32
    }    

    fn square_resize_and_gray(img: &DynamicImage, size: u32) 
        -> ImageBuf<Luma<u8>> {
        let small = resize(img, size, size, FILTER_TYPE);
        grayscale(&small)
    }

    fn fast_hash(img: &DynamicImage, hash_size: u32) -> Bitv {
        let temp = ImageHash::square_resize_and_gray(img, hash_size);

        let hash_values: Vec<u8> = temp.pixels().map(|(_, _, x)| x.channel())
            .collect();

        let hash_sq = (hash_size * hash_size) as uint;

        let mean = hash_values.iter().fold(0u, |b, &a| a as uint + b) 
            / hash_sq;

        hash_values.move_iter().map(|x| x as uint >= mean).collect()
    }

    fn dct_hash(img: &DynamicImage, hash_size: u32) -> Bitv {
        let large_size = hash_size * 4;

        // We take a bigger resize than fast_hash, 
        // then we only take the lowest corner of the DCT
        let temp = ImageHash::square_resize_and_gray(img, large_size);

        // Our hash values are converted to doubles for the DCT
        let hash_values: Vec<f64> = temp.pixels()
            .map(|(_, _, x)| x.channel() as f64).collect();

        let dct = dct_2d(hash_values.as_slice(),
            large_size as uint, large_size as uint);

        let original = (large_size as uint, large_size as uint);
        let new = (hash_size as uint, hash_size as uint);

        let cropped_dct = crop_dct(dct, original, new);

        let mean = cropped_dct.iter().fold(0f64, |b, &a| a + b) 
            / (hash_size * hash_size) as f64;

        cropped_dct.move_iter().map(|x| x >= mean).collect()
    }    

    pub fn hash(img: &DynamicImage, hash_size: u32, fast: bool) -> ImageHash {
        let hash = if fast { 
            ImageHash::fast_hash(img, hash_size)   
        } else { 
            ImageHash::dct_hash(img, hash_size)             
        };

        assert!((hash_size * hash_size) as uint == hash.len());

        ImageHash {
            size: hash_size * hash_size,
            bitv: hash,
        }
    }
}

