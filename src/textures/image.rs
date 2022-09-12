//! An image-backed texture mapping

use crate::{color::Color, textures::Texture};

/// An image-based texture
#[derive(Debug)]
pub struct ImageMap {
    /// The image buffer used as the texture
    image: image::RgbImage,
}

impl ImageMap {
    /// Creates a new [ImageMap]
    ///
    /// Loads the image located at `file_path`:
    /// * if successful, holds the decoded [image::RgbImage] in an Option
    /// * on error, holds a default "missing" texture
    ///
    /// Missing texture sourced from [The GMod fandom wiki](https://gmod.fandom.com/wiki/Missing_textures),
    /// available under CC-BY-SA
    pub fn new(file_path: std::path::PathBuf) -> Self {
        use image::io::Reader;
        let dyn_img = match Reader::open(&file_path) {
            Ok(file_reader) => file_reader.decode(),
            Err(_) => {
                // TODO log file read error
                // Adapted from [image::io::Reader] usage page
                use std::io::Cursor;
                Reader::new(Cursor::new(include_bytes!("../../resources/default.png")))
                    .with_guessed_format()
                    .expect("We should never fail with binary Cursor reads")
                    .decode()
            }
        };

        Self {
            image: dyn_img
                .map(|dyn_img| dyn_img.into_rgb8())
                .expect("We should always have some image data to fall back on"),
        }
    }
}

impl Texture for ImageMap {
    fn color(&self, u: f32, v: f32, _point: glam::Vec3A) -> Color {
        let u = u.clamp(0.0, 1.0);
        let v = 1.0 - v.clamp(0.0, 1.0);

        let i = u * self.image.width() as f32;
        let j = v * self.image.height() as f32;

        let i = (i as u32).clamp(0, self.image.width() - 1);
        let j = (j as u32).clamp(0, self.image.height() - 1);

        // let color_scale = 1.0 / 255.0;
        let pixel = self.image[(i, j)];
        Color::from(pixel)
    }
}
