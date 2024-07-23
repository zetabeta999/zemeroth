use std::{fmt::Debug, iter::repeat};

use num::{Num, Signed};
use serde::{Deserialize, Serialize};

/// Cube coordinates
/// <http://www.redblobgames.com/grids/hexagons/#coordinates-cube>
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PosCube<T: Debug + Copy = i32> {
    pub x: T,
    pub y: T,
    pub z: T,
}

/// Axial coordinates
/// <http://www.redblobgames.com/grids/hexagons/#coordinates-axial>
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Hash, Eq)]
pub struct PosHex<T: Debug + Copy = i32> {
    /// column
    pub q: T,

    /// row
    pub r: T,
}

impl<T: Debug + Copy> PosHex<T> {
    pub fn new(q: T, r: T) -> Self {
        Self { q, r }
    }
}

pub fn hex_to_cube<N: Num + Copy + Debug + Signed>(hex: PosHex<N>) -> PosCube<N> {
    PosCube {
        x: hex.q,
        y: -hex.q - hex.r,
        z: hex.r,
    }
}

pub fn cube_to_hex<T: Debug + Copy>(cube: PosCube<T>) -> PosHex<T> {
    PosHex {
        q: cube.x,
        r: cube.z,
    }
}

pub fn hex_round(hex: PosHex<f32>) -> PosHex {
    cube_to_hex(cube_round(hex_to_cube(hex)))
}

/// <http://www.redblobgames.com/grids/hexagons/#rounding>
pub fn cube_round(cube: PosCube<f32>) -> PosCube {
    let mut rx = cube.x.round();
    let mut ry = cube.y.round();
    let mut rz = cube.z.round();
    let x_diff = (rx - cube.x).abs();
    let y_diff = (ry - cube.y).abs();
    let z_diff = (rz - cube.z).abs();
    if x_diff > y_diff && x_diff > z_diff {
        rx = -ry - rz;
    } else if y_diff > z_diff {
        ry = -rx - rz;
    } else {
        rz = -rx - ry;
    }
    PosCube {
        x: rx as i32,
        y: ry as i32,
        z: rz as i32,
    }
}

pub fn distance_cube(a: PosCube, b: PosCube) -> i32 {
    ((a.x - b.x).abs() + (a.y - b.y).abs() + (a.z - b.z).abs()) / 2
}

pub fn distance_hex(a: PosHex, b: PosHex) -> i32 {
    distance_cube(hex_to_cube(a), hex_to_cube(b))
}

fn is_inboard(radius: i32, pos: PosHex) -> bool {
    let origin = PosCube { x: 0, y: 0, z: 0 };
    distance_cube(origin, hex_to_cube(pos)) <= radius
}

#[derive(Clone, Debug)]
pub struct HexIter {
    cursor: PosHex,
    radius: i32,
}

impl HexIter {
    fn new(radius: i32) -> Self {
        let mut iter = Self {
            cursor: PosHex {
                q: -radius,
                r: -radius,
            },
            radius,
        };
        iter.inc_cursor_with_hex_bounds();
        iter
    }

    fn inc_cursor(&mut self) {
        self.cursor.q += 1;
        if self.cursor.q > self.radius {
            self.cursor.q = -self.radius;
            self.cursor.r += 1;
        }
    }

    fn inc_cursor_with_hex_bounds(&mut self) {
        self.inc_cursor();
        while !is_inboard(self.radius, self.cursor) && self.cursor.r < self.radius + 1 {
            self.inc_cursor();
        }
    }
}

impl Iterator for HexIter {
    type Item = PosHex;

    fn next(&mut self) -> Option<PosHex> {
        if self.cursor.r > self.radius {
            None
        } else {
            let current = self.cursor;
            self.inc_cursor_with_hex_bounds();
            Some(current)
        }
    }
}

pub fn radius_to_diameter(radius: i32) -> i32 {
    radius * 2 + 1
}

///
///     [-1, 0]  [0, -1]
/// [-1, 1]  [0, 0]  [1, -1]
///     [ 0, 1]  [ 1, 0]
///
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HexMap<T: Copy + Debug> {
    pub tiles: Vec<T>,
    pub size: i32,
    pub radius: i32,
}

impl<T: Copy + Default + Debug> HexMap<T> {
    pub fn new(radius: i32) -> Self {
        let size = radius * 2 + 1;
        let tiles_count = (size * size) as usize;
        let tiles = repeat(Default::default()).take(tiles_count).collect();
        Self {
            tiles,
            size,
            radius,
        }
    }

    pub fn radius(&self) -> i32 {
        self.radius
    }

    pub fn height(&self) -> i32 {
        radius_to_diameter(self.radius())
    }

    pub fn iter(&self) -> HexIter {
        HexIter::new(self.radius)
    }

    pub fn is_inboard(&self, pos: PosHex) -> bool {
        is_inboard(self.radius, pos)
    }

    fn hex_to_index(&self, hex: PosHex) -> usize {
        let i = (hex.r + self.radius) + (hex.q + self.radius) * self.size;
        i as usize
    }

    pub fn tile(&self, pos: PosHex) -> T {
        assert!(self.is_inboard(pos));
        self.tiles[self.hex_to_index(pos)]
    }

    pub fn set_tile(&mut self, pos: PosHex, tile: T) {
        assert!(self.is_inboard(pos));
        let index = self.hex_to_index(pos);
        self.tiles[index] = tile;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum Dir {
    SouthEast,
    East,
    NorthEast,
    NorthWest,
    West,
    SouthWest,
}

/// <http://www.redblobgames.com/grids/hexagons/#neighbors-axial>
const DIR_TO_POS_DIFF: [[i32; 2]; 6] = [[1, 0], [1, -1], [0, -1], [-1, 0], [-1, 1], [0, 1]];

impl Dir {
    pub fn from_int(n: i32) -> Self {
        assert!((0..6).contains(&n));
        let dirs = [
            Dir::SouthEast,
            Dir::East,
            Dir::NorthEast,
            Dir::NorthWest,
            Dir::West,
            Dir::SouthWest,
        ];
        dirs[n as usize]
    }

    pub fn to_int(self) -> i32 {
        match self {
            Dir::SouthEast => 0,
            Dir::East => 1,
            Dir::NorthEast => 2,
            Dir::NorthWest => 3,
            Dir::West => 4,
            Dir::SouthWest => 5,
        }
    }

    pub fn get_dir_from_to(from: PosHex, to: PosHex) -> Self {
        assert_eq!(distance_hex(from, to), 1);
        let diff = [to.q - from.q, to.r - from.r];
        for dir in dirs() {
            if diff == DIR_TO_POS_DIFF[dir.to_int() as usize] {
                return dir;
            }
        }
        panic!("impossible positions: {:?}, {:?}", from, to);
    }

    pub fn get_neighbor_pos(pos: PosHex, dir: Self) -> PosHex {
        let diff = DIR_TO_POS_DIFF[dir.to_int() as usize];
        PosHex {
            q: pos.q + diff[0],
            r: pos.r + diff[1],
        }
    }

    pub fn get_dir_pos(from: PosHex, to: PosHex) -> PosHex {
        assert_eq!(distance_hex(from, to), 1);
        let diff = [to.q - from.q, to.r - from.r];
        for dir in dirs() {
            if diff == DIR_TO_POS_DIFF[dir.to_int() as usize] {
                return PosHex {
                    q: to.q + diff[0],
                    r: to.r + diff[1],
                };
            }
        }
        panic!("impossible positions: {:?}, {:?}", from, to);
    }
}

#[derive(Clone, Debug)]
pub struct DirIter {
    index: i32,
}

pub fn dirs() -> DirIter {
    DirIter { index: 0 }
}

impl Iterator for DirIter {
    type Item = Dir;

    fn next(&mut self) -> Option<Dir> {
        let max = DIR_TO_POS_DIFF.len() as i32;
        let next_dir = if self.index >= max {
            None
        } else {
            Some(Dir::from_int(self.index))
        };
        self.index += 1;
        next_dir
    }
}

#[cfg(test)]
mod tests {
    use crate::map::HexMap;

    #[test]
    fn test_map_height() {
        let map: HexMap<u8> = HexMap::new(3);
        let height = map.height();
        assert_eq!(height, 7);
    }
}
