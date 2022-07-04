use std::{fmt::Debug, rc::Rc};

use glam::Vec3;

use crate::color::Color;

pub trait Texture {
    fn color(&self, u: f32, v: f32, point: Vec3) -> Color;
}

impl Debug for dyn Texture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Texture with Color {:?}",
            self.color(0.0, 0.0, Vec3::ZERO)
        )
    }
}

impl Texture for Color {
    fn color(&self, _u: f32, _v: f32, _point: Vec3) -> Color {
        *self
    }
}

pub struct Checkered {
    pub even: Rc<dyn Texture>,
    pub odd: Rc<dyn Texture>,
}

impl Checkered {
    pub fn new(o: &Rc<dyn Texture>, e: &Rc<dyn Texture>) -> Self {
        Self {
            even: e.clone(),
            odd: o.clone(),
        }
    }
}

impl Texture for Checkered {
    fn color(&self, u: f32, v: f32, point: Vec3) -> Color {
        let sin_x = (point * 10.0).x.sin();
        let sin_y = (point * 10.0).y.sin();
        let sin_z = (point * 10.0).z.sin();

        if sin_x * sin_y * sin_z < 0.0 {
            self.odd.color(u, v, point)
        } else {
            self.even.color(u, v, point)
        }
    }
}
