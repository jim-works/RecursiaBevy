use std::f32::consts::PI;

use bevy::prelude::*;

use super::{FlattenRef, FlattenRefMut};

#[derive(Clone)]
pub struct VolumeIterator {
    x_len: i32,
    y_len: i32,
    z_len: i32,
    x_i: i32,
    y_i: i32,
    z_i: i32,
    done: bool, //this is ugly but i'm tired
}

impl VolumeIterator {
    pub fn new(x: u32, y: u32, z: u32) -> Self {
        Self {
            x_len: x as i32,
            y_len: y as i32,
            z_len: z as i32,
            x_i: 0,
            y_i: 0,
            z_i: 0,
            done: x == 0 || y == 0 || z == 0,
        }
    }

    pub fn from_volume(volume: Volume) -> impl Iterator<Item = IVec3> + Clone {
        let size = volume.max_corner - volume.min_corner;
        Self {
            x_len: size.x,
            y_len: size.y,
            z_len: size.z,
            x_i: 0,
            y_i: 0,
            z_i: 0,
            done: size.x <= 0 || size.y <= 0 || size.z <= 0,
        }
        .map(move |offset| volume.min_corner + offset)
    }
}

impl Iterator for VolumeIterator {
    type Item = IVec3;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        let ret = Some(IVec3::new(self.x_i, self.y_i, self.z_i));
        self.x_i += 1;
        if self.x_i >= self.x_len {
            self.y_i += 1;
            self.x_i = 0;
        }
        if self.y_i >= self.y_len {
            self.z_i += 1;
            self.y_i = 0;
        }
        if self.z_i >= self.z_len {
            self.done = true;
        }
        ret
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Volume {
    pub min_corner: IVec3,
    pub max_corner: IVec3,
}

impl Volume {
    //returns true if min <= other min and max >= other max.
    //contains itself!
    pub fn contains(&self, other: Volume) -> bool {
        (self.min_corner.x <= other.min_corner.x
            && self.min_corner.y <= other.min_corner.y
            && self.min_corner.z <= other.min_corner.z)
            && (self.max_corner.x >= other.max_corner.x
                && self.max_corner.y >= other.max_corner.y
                && self.max_corner.z >= other.max_corner.z)
    }

    pub fn intersects(&self, other: Volume) -> bool {
        (self.min_corner.x <= other.max_corner.x && self.max_corner.x >= other.min_corner.x)
            && (self.min_corner.y <= other.max_corner.y && self.max_corner.y >= other.min_corner.y)
            && (self.min_corner.z <= other.max_corner.z && self.max_corner.z >= other.min_corner.z)
    }

    pub fn volume(&self) -> i32 {
        (self.max_corner.x - self.min_corner.x)
            * (self.max_corner.y - self.min_corner.y)
            * (self.max_corner.z - self.min_corner.z)
    }

    pub fn size(&self) -> IVec3 {
        self.max_corner - self.min_corner
    }

    pub fn center(&self) -> Vec3 {
        self.min_corner.as_vec3() + self.size().as_vec3() / 2.0
    }

    pub fn new(min_corner: IVec3, max_corner_exclusive: IVec3) -> Self {
        Volume {
            min_corner,
            max_corner: max_corner_exclusive,
        }
    }

    pub fn new_inclusive(min_corner: IVec3, max_corner_inclusive: IVec3) -> Self {
        Volume {
            min_corner,
            max_corner: max_corner_inclusive + IVec3::new(1, 1, 1),
        }
    }

    pub fn iter(self) -> impl Iterator<Item = IVec3> + Clone {
        VolumeIterator::from_volume(self)
    }
}

pub struct VolumeContainer<T> {
    blocks: Vec<Option<T>>,
    volume: Volume,
    size: IVec3,
}

impl<'a, T> VolumeContainer<T> {
    pub fn new(volume: Volume) -> Self {
        let mut vec = Vec::with_capacity(volume.volume() as usize);
        vec.resize_with(volume.volume() as usize, || None);
        Self {
            blocks: vec,
            volume,
            size: volume.max_corner - volume.min_corner,
        }
    }

    pub fn volume(&self) -> Volume {
        self.volume
    }

    pub fn size(&self) -> IVec3 {
        self.size
    }

    pub fn iter(&'a self) -> impl Iterator<Item = (IVec3, Option<&'a T>)> + Clone + 'a {
        self.volume.iter().map(|pos| (pos, self.get(pos)))
    }

    //clears blocks, and reuses buffer for new volume, expanding if needed
    pub fn recycle(&mut self, volume: Volume) {
        self.volume = volume;
        self.size = volume.max_corner - volume.min_corner;
        self.blocks.clear();
        self.blocks.resize_with(volume.volume() as usize, || None);
    }

    pub fn get(&self, mut index: IVec3) -> Option<&T> {
        index -= self.volume.min_corner;
        self.blocks
            .get((index.x + index.y * self.size.x + index.z * self.size.x * self.size.y) as usize)
            .flatten()
    }

    pub fn get_mut(&mut self, mut index: IVec3) -> Option<&mut T> {
        index -= self.volume.min_corner;
        self.blocks
            .get_mut(
                (index.x + index.y * self.size.x + index.z * self.size.x * self.size.y) as usize,
            )
            .flatten()
    }

    pub fn set(&mut self, mut index: IVec3, value: Option<T>) {
        index -= self.volume.min_corner;
        self.blocks
            [(index.x + index.y * self.size.x + index.z * self.size.x * self.size.y) as usize] =
            value;
    }
}

pub trait AxisIter<T> {
    fn axis_iter(self) -> impl Iterator<Item = T>;
}

impl AxisIter<f32> for Vec3 {
    fn axis_iter(self) -> impl Iterator<Item = f32> {
        self.to_array().into_iter()
    }
}

impl AxisIter<i32> for IVec3 {
    fn axis_iter(self) -> impl Iterator<Item = i32> {
        [self.x, self.y, self.z].into_iter()
    }
}

pub trait AxisMap<Elem, ResultElem, Result = Self> {
    fn axis_map(self, f: impl FnMut(Elem) -> ResultElem) -> Result;
}

impl AxisMap<f32, f32> for Vec3 {
    fn axis_map(self, mut f: impl FnMut(f32) -> f32) -> Self {
        Vec3::new((f)(self.x), (f)(self.y), (f)(self.z))
    }
}

impl AxisMap<(f32, f32), f32, Vec3> for (Vec3, Vec3) {
    fn axis_map(self, mut f: impl FnMut((f32, f32)) -> f32) -> Vec3 {
        Vec3::new(
            (f)((self.0.x, self.1.x)),
            (f)((self.0.y, self.1.y)),
            (f)((self.0.z, self.1.z)),
        )
    }
}

impl AxisMap<(f32, f32, f32), f32, Vec3> for (Vec3, Vec3, Vec3) {
    fn axis_map(self, mut f: impl FnMut((f32, f32, f32)) -> f32) -> Vec3 {
        Vec3::new(
            (f)((self.0.x, self.1.x, self.2.x)),
            (f)((self.0.y, self.1.y, self.2.y)),
            (f)((self.0.z, self.1.z, self.2.z)),
        )
    }
}

impl AxisMap<i32, i32> for IVec3 {
    fn axis_map(self, mut f: impl FnMut(i32) -> i32) -> Self {
        IVec3::new((f)(self.x), (f)(self.y), (f)(self.z))
    }
}

//usese fibonacci sphere algorithm https://arxiv.org/pdf/0912.4540.pdf
//generates points evenly spaced on unit sphere
pub fn even_distribution_on_sphere(samples: u32) -> impl Iterator<Item = Vec3> {
    let phi = PI * (5.0_f32.sqrt() - 1.); //golden angle in radians
    (0..samples).map(move |i| {
        let y = 1.0 - (i as f32 / (samples - 1) as f32) * 2.0;
        let radius = (1.0 - y * y).sqrt();
        let theta = phi * i as f32;
        Vec3::new(theta.cos() * radius, y, theta.sin() * radius)
    })
}
