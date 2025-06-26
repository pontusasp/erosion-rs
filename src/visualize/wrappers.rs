use crate::heightmap::Heightmap;
use crate::visualize::heightmap_to_image;
use bracket_noise::prelude::{FractalType, NoiseType};
use egui::ColorImage;
use serde::{Deserialize, Serialize};
use std::rc::Rc;

#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum NoiseTypeWrapper {
    Value,
    ValueFractal,
    Perlin,
    PerlinFractal,
    Simplex,
    SimplexFractal,
    Cellular,
    WhiteNoise,
    Cubic,
    CubicFractal,
}

impl From<NoiseType> for NoiseTypeWrapper {
    fn from(item: NoiseType) -> Self {
        match item {
            NoiseType::Value => NoiseTypeWrapper::Value,
            NoiseType::ValueFractal => NoiseTypeWrapper::ValueFractal,
            NoiseType::Perlin => NoiseTypeWrapper::Perlin,
            NoiseType::PerlinFractal => NoiseTypeWrapper::PerlinFractal,
            NoiseType::Simplex => NoiseTypeWrapper::Simplex,
            NoiseType::SimplexFractal => NoiseTypeWrapper::SimplexFractal,
            NoiseType::Cellular => NoiseTypeWrapper::Cellular,
            NoiseType::WhiteNoise => NoiseTypeWrapper::WhiteNoise,
            NoiseType::Cubic => NoiseTypeWrapper::Cubic,
            NoiseType::CubicFractal => NoiseTypeWrapper::CubicFractal,
        }
    }
}

impl From<NoiseTypeWrapper> for NoiseType {
    fn from(item: NoiseTypeWrapper) -> Self {
        match item {
            NoiseTypeWrapper::Value => NoiseType::Value,
            NoiseTypeWrapper::ValueFractal => NoiseType::ValueFractal,
            NoiseTypeWrapper::Perlin => NoiseType::Perlin,
            NoiseTypeWrapper::PerlinFractal => NoiseType::PerlinFractal,
            NoiseTypeWrapper::Simplex => NoiseType::Simplex,
            NoiseTypeWrapper::SimplexFractal => NoiseType::SimplexFractal,
            NoiseTypeWrapper::Cellular => NoiseType::Cellular,
            NoiseTypeWrapper::WhiteNoise => NoiseType::WhiteNoise,
            NoiseTypeWrapper::Cubic => NoiseType::Cubic,
            NoiseTypeWrapper::CubicFractal => NoiseType::CubicFractal,
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum FractalTypeWrapper {
    FBM,
    Billow,
    RigidMulti,
}

impl From<FractalType> for FractalTypeWrapper {
    fn from(value: FractalType) -> Self {
        match value {
            FractalType::FBM => FractalTypeWrapper::FBM,
            FractalType::Billow => FractalTypeWrapper::Billow,
            FractalType::RigidMulti => FractalTypeWrapper::RigidMulti,
        }
    }
}

impl From<FractalTypeWrapper> for FractalType {
    fn from(value: FractalTypeWrapper) -> Self {
        match value {
            FractalTypeWrapper::FBM => FractalType::FBM,
            FractalTypeWrapper::Billow => FractalType::Billow,
            FractalTypeWrapper::RigidMulti => FractalType::RigidMulti,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HeightmapTexture {
    #[serde(skip)]
    pub image: Option<Rc<ColorImage>>,
    pub heightmap: Rc<Heightmap>,
}

impl HeightmapTexture {
    pub fn new(heightmap: Rc<Heightmap>, image: Option<Rc<ColorImage>>) -> Self {
        Self {
            image,
            heightmap,
        }
    }

    pub fn get_or_generate(&self) -> Rc<ColorImage> {
        if let Some(image) = &self.image {
            Rc::clone(image)
        } else {
            Rc::new(heightmap_to_image(&self.heightmap))
        }
    }

    pub fn get_and_generate_cache(&mut self) -> Rc<ColorImage> {
        let image = self.get_or_generate();
        self.image = Some(Rc::clone(&image));
        image
    }
}

impl From<&Rc<Heightmap>> for HeightmapTexture {
    fn from(value: &Rc<Heightmap>) -> Self {
        let image = heightmap_to_image(value);
        Self {
            image: Some(Rc::new(image)),
            heightmap: Rc::clone(value),
        }
    }
}

impl From<Heightmap> for HeightmapTexture {
    fn from(value: Heightmap) -> Self {
        let image = heightmap_to_image(&value);
        Self {
            image: Some(Rc::new(image)),
            heightmap: Rc::new(value),
        }
    }
}

impl From<HeightmapTexture> for Rc<ColorImage> {
    fn from(value: HeightmapTexture) -> Self {
        if let Some(image) = value.image {
            image
        } else {
            value.get_or_generate()
        }
    }
}
