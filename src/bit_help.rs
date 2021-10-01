use std::cmp::min;
use std::iter::{Copied, Chain};

#[derive(Clone, Copy)]
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
        if 0 <= x && x < 7 && 0 <= y && y < 7 {
            return Some(index_to_place(coord_to_index((dx, dy))));
        }
        return None;
    }

    pub fn adj() -> Copied<std::slice::Iter<'static, Dir>> {
        static adjs: [Dir; 4] = [Dir::North, Dir::South, Dir::East, Dir::West];
        return adjs.iter().copied();
    }

    pub fn diag() -> Copied<std::slice::Iter<'static, Dir>> {
        static diag: [Dir; 4] = [Dir::NorthEast, Dir::NorthWest, Dir::SouthEast, Dir::SouthWest];
        return diag.iter().copied();
    }

    pub fn all() -> Chain<Copied<std::slice::Iter<'static, Dir>>, Copied<std::slice::Iter<'static, Dir>>> {
        return Dir::adj().chain(Dir::diag());
    }

}

pub struct U64Iterator {
    cur: u64
}

impl Iterator for U64Iterator {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        return if self.cur == 0 {
            None
        } else {
            let res = Some(self.cur.leading_zeros() as i32);
            self.cur &= self.cur - 1;
            res
        }
    }
}

pub fn iter_u64(board: u64) -> U64Iterator {
    U64Iterator { cur: board }
}

pub fn index_to_place(index: i32) -> u64 {
    return 1_u64 << index;
}

pub fn index(place: u64) -> i32 {
    place.leading_zeros() as i32
}

pub fn coord_to_index(coord: (i32, i32)) -> i32 {
    return coord.0 * 8 + coord.1
}

pub fn index_to_coord(index: i32) -> (i32, i32) {
    return (index / 8, index % 8)
}

pub fn place_to_coord(place: u64) -> (i32, i32) {
    return index_to_coord(index(place));
}

pub fn ray(index: i32, dir: Dir) -> u64 {
    let start = index_to_place(index);
    let (n, e, w, s) = (
        (index / 8) - 1,
        7 - (index / 8),
        (index % 8) - 1,
        7 - (index % 8)
    );
    match dir {
        Dir::North => _ray(start, n, |x| x >> 8),
        Dir::South => _ray(start, e, |x| x << 8),
        Dir::East => _ray(start, w, |x| x >> 1),
        Dir::West => _ray(start, s, |x| x << 1),
        Dir::NorthEast => _ray(start, min(n, e), |x| x >> 7),
        Dir::NorthWest => _ray(start, min(n, w), |x| x >> 9),
        Dir::SouthEast => _ray(start, min(s, e), |x| x << 9),
        Dir::SouthWest => _ray(start, min(s, w), |x| x << 7),
    }
}

fn _ray(mut start: u64, cnt: i32, func: fn(u64) -> u64) -> u64 {
    let mut ray = 0_u64;
    for _ in 0..cnt {
        start = func(start);
        ray |= start;
    }
    ray
}

pub fn ray_until_blocker(index:i32, blockers: u64, dir: Dir) -> u64 {
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
        res &= !(ray(first_blocker_index as i32, dir))
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
        for i in 0..-1 {
            panic!("WHAT")
        }
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
