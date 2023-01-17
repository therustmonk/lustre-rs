//! Render an image given a [Camera] and a [Hittable].

use glam::Vec3A;
use rand::{rngs::SmallRng, Rng, SeedableRng};

use image::{DynamicImage, ImageFormat};

#[cfg(feature = "parallel")]
use {indicatif::ParallelProgressIterator, rayon::prelude::*};

#[cfg(not(feature = "parallel"))]
use indicatif::ProgressIterator;

use crate::{
    camera::Camera, color::Color, hittables::Hittable, tree::Tree, utils::progress::get_progressbar,
};

/// Stores render context values such as image dimensions and scene geometry
pub struct RenderContext {
    image_width: u32,
    image_height: u32,
    samples_per_pixel: u32,
    bounce_depth: u16,
    camera: Camera,
    geometry: std::sync::Arc<dyn Hittable>,
    output_hdr: bool,
}

impl RenderContext {
    pub fn from_arguments(args: &crate::cli::Arguments, rng: &mut impl Rng) -> Self {
        let (geometry, camera, (width, height)) =
            crate::scenes::get_scene(args.image_width, args.scene, rng);
        let geometry = Tree::new(
            geometry,
            camera.shutter_open_time,
            camera.shutter_close_time,
        );

        let output_hdr = matches!(
            ImageFormat::from_path(&args.output).unwrap(),
            ImageFormat::OpenExr
        );

        Self {
            image_width: width,
            image_height: height,
            camera,
            geometry: geometry.wrap(),
            bounce_depth: args.bounce_depth,
            samples_per_pixel: args.samples_per_pixel,
            output_hdr,
        }
    }

    /// Calculates the total color value of the pixel at image coordinates (`x`, `y`)
    ///
    /// Uses the provided [Camera] to translate the image coordinates
    /// to world space coordinates, then computes the color value
    #[inline]
    fn compute_pixel_v(&self, x: u32, y: u32, rng: &mut impl Rng) -> Vec3A {
        // convert buffer indices to viewport coordinates
        let offset_u: f32 = rng.gen();
        let offset_v: f32 = rng.gen();
        let u = (x as f32 + offset_u) / (self.image_width - 1) as f32;
        let v = ((self.image_height - y) as f32 + offset_v) / (self.image_height - 1) as f32;

        // trace ray
        let contrib = self.camera.get_ray(u, v, rng).shade(
            &self.geometry,
            self.bounce_depth,
            self.camera.bg_color,
            rng,
        );
        Vec3A::from(contrib)
    }

    /// Generates an image from the given scene.
    ///
    /// A scene consists of a [Camera] and some [Hittable].
    /// This functions outputs its progress to the commandline.
    pub fn render(&self) -> DynamicImage {
        let progress_bar = get_progressbar((self.image_height * self.image_width) as u64)
            .with_prefix("Generating pixels");

        // Allocate image buffer
        // default to f32 to keep hdr data until write time
        let mut img_buf = image::Rgb32FImage::new(self.image_width, self.image_height);

        // Generate image
        #[cfg(feature = "parallel")]
        img_buf
            .enumerate_pixels_mut()
            .par_bridge()
            .progress_with(progress_bar)
            .for_each(|(x, y, pixel)| {
                // map reduce N samples into single Vec3A
                let mut color_v: Vec3A = (0..self.samples_per_pixel)
                    .into_par_iter()
                    .map_init(
                        // from_rng(...) gives Result, can assume it won't fail
                        || SmallRng::from_rng(&mut rand::thread_rng()).unwrap(),
                        // current sample # doesn't matter, ignore
                        |rng, _| self.compute_pixel_v(x, y, rng),
                    )
                    .sum();

                // Account for number of samples
                color_v /= self.samples_per_pixel as f32;

                // "gamma" correction
                if !self.output_hdr {
                    color_v = color_v.powf(0.5); // sqrt
                }

                // modify pixel with generated color value
                *pixel = Color::new(color_v).into();
            });
        #[cfg(not(feature = "parallel"))]
        img_buf
            .enumerate_pixels_mut()
            .progress_with(progress_bar)
            .for_each(|(x, y, pixel)| {
                // map reduce N samples into single Vec3A
                let mut color_v: Vec3A = (0..self.samples_per_pixel)
                    .map(
                        // current sample # doesn't matter, ignore
                        |_| {
                            let rng = &mut SmallRng::from_rng(&mut rand::thread_rng()).unwrap();
                            self.compute_pixel_v(x, y, rng)
                        },
                    )
                    .sum();

                // Account for number of samples
                color_v /= self.samples_per_pixel as f32;

                // "gamma" correction
                if !self.output_hdr {
                    color_v = color_v.powf(0.5); // sqrt
                }

                // modify pixel with generated color value
                *pixel = Color::new(color_v).into();
            });

        if self.output_hdr {
            DynamicImage::ImageRgb32F(img_buf)
        } else {
            use image::buffer::ConvertBuffer;
            DynamicImage::ImageRgb8(img_buf.convert())
        }
    }
}
