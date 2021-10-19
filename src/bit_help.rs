use std::cmp::min;
use std::iter::{Copied, Chain};

#[derive(Debug, Clone, Copy)]
pub enum Dir {North, South, East, West, NorthEast, NorthWest, SouthEast, SouthWest}

impl Dir {
    pub fn mv(&self, place: u64) -> Option<u64> {
        let (x, y) = place_to_coord(place);
        let (dx, dy) = match *self {
            Dir::North => (x, y - 1),
            Dir::South => (x, y + 1),
            Dir::East => (x + 1, y),
            Dir::West => (x - 1, y),
            Dir::NorthEast => (x + 1, y - 1),
            Dir::NorthWest => (x - 1, y - 1),
            Dir::SouthEast => (x + 1, y + 1),
            Dir::SouthWest => (x - 1, y + 1)
        };
        if 0 <= dx && dx <= 7 && 0 <= dy && dy <= 7 {
            return Some(index_to_place(coord_to_index((dx, dy))));
        }
        return None;
    }

    pub fn flip(&self) -> Dir {
        match *self {
            Dir::North => Dir::South,
            Dir::South => Dir::North,
            Dir::East => Dir::West,
            Dir::West => Dir::East,
            Dir::NorthEast => Dir::SouthWest,
            Dir::NorthWest => Dir::SouthEast,
            Dir::SouthEast => Dir::NorthWest,
            Dir::SouthWest => Dir::NorthEast
        }
    }

    pub fn adj() -> Copied<std::slice::Iter<'static, Dir>> {
        static ADJS: [Dir; 4] = [Dir::North, Dir::South, Dir::East, Dir::West];
        return ADJS.iter().copied();
    }

    pub fn diag() -> Copied<std::slice::Iter<'static, Dir>> {
        static DIAG: [Dir; 4] = [Dir::NorthEast, Dir::NorthWest, Dir::SouthEast, Dir::SouthWest];
        return DIAG.iter().copied();
    }

    pub fn all() -> Chain<Copied<std::slice::Iter<'static, Dir>>, Copied<std::slice::Iter<'static, Dir>>> {
        return Dir::adj().chain(Dir::diag());
    }

}

pub struct IndexIterator {
    cur: u64
}

impl Iterator for IndexIterator {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        return if self.cur == 0 {
            None
        } else {
            let res = Some(self.cur.trailing_zeros() as usize);
            self.cur &= self.cur - 1;
            res
        }
    }
}

pub fn iter_index(board: u64) -> IndexIterator {
    IndexIterator { cur: board }
}

pub struct PlaceIterator {
    cur: u64
}

impl Iterator for PlaceIterator {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        return if self.cur == 0 {
            None
        } else {
            let prev = self.cur;
            self.cur &= self.cur - 1;
            Some(prev ^ self.cur)
        }
    }
}

pub fn iter_place(board: u64) -> PlaceIterator {
    PlaceIterator { cur: board }
}

pub fn index_to_place(index: usize) -> u64 {
    return 1_u64 << index;
}

pub fn index(place: u64) -> usize { place.trailing_zeros() as usize }

pub fn coord_to_index(coord: (i32, i32)) -> usize { (coord.1 * 8 + coord.0) as usize }

pub fn index_to_coord(index: usize) -> (i32, i32) {
    return (index as i32 % 8, index as i32 / 8)
}

pub fn place_to_coord(place: u64) -> (i32, i32) {
    return index_to_coord(index(place));
}


pub fn ray(index: usize, dir: Dir) -> u64 {
    _ray(index, dir, 0)
}

pub fn _ray(index: usize, dir: Dir, edge_buffer: i32) -> u64 {
    // not including index.
    let start = index_to_place(index);
    let i = index as i32;
    let (n, s, e, w) = (
        (i / 8) - edge_buffer,
        7 - (i / 8) - edge_buffer,
        7 - (i % 8) - edge_buffer,
        (i % 8) - edge_buffer
    );
    match dir {
        Dir::North => __ray(start, n, |x| x >> 8),
        Dir::South => __ray(start, s, |x| x << 8),
        Dir::East => __ray(start, e, |x| x << 1),
        Dir::West => __ray(start, w, |x| x >> 1),
        Dir::NorthEast => __ray(start, min(n, e), |x| x >> 7),
        Dir::NorthWest => __ray(start, min(n, w), |x| x >> 9),
        Dir::SouthEast => __ray(start, min(s, e), |x| x << 9),
        Dir::SouthWest => __ray(start, min(s, w), |x| x << 7),
    }
}

fn __ray(mut start: u64, cnt: i32, func: fn(u64) -> u64) -> u64 {
    let mut ray = 0_u64;
    for _ in 0..cnt {
        start = func(start);
        ray |= start;
    }
    ray
}

pub fn ray_until_blocker(index: usize, blockers: u64, dir: Dir) -> u64 {
    // not including index, including blocker.
    let mut res = ray(index, dir);
    if res & blockers != 0 {
        let first_blocker_index = match dir {
            Dir::NorthEast |
            Dir::NorthWest |
            Dir::North |
            Dir::West => 63 - (res & blockers).leading_zeros(),
            Dir::SouthEast |
            Dir::SouthWest |
            Dir::South |
            Dir::East => (res & blockers).trailing_zeros(),
        };
        res &= !(ray(first_blocker_index as usize, dir))
    }
    res
}


// Tests
pub struct X {
    thing: u64
}

impl X {
    pub fn get_mut(&mut self) -> &mut u64 {
        &mut self.thing
    }
}

#[cfg(test)]
mod tests {
    use crate::bit_help::{X, index_to_place};


    #[test]
    fn test_stuff() {
        let mut v = vec![true, false];
        v.sort();
        println!("{:?}", v);
    }

    #[test]
    fn test_place() {
        for i in 0..64 {
            assert_eq!(index_to_place(i), 2_u64.pow(i as u32));
        }
    }

    #[test]
    fn test_bit_shift() {
        assert_eq!(8_u64 >> 1, 4)
    }

    #[test]
    fn test_reduce() {
        let v = vec![1, 2, 3];
        v.iter().cloned().reduce(|a, b| a | b).unwrap();
        assert_eq!(v, vec![1, 2 , 3])
    }

    #[test]
    fn stuff() {
        let mut y = X { thing: 2};
        assert_eq!(y.thing, 2);
        *y.get_mut() |= 4;
        assert_eq!(y.thing, 6);

    }
}
