

pub fn place(index: i32) -> u64 {
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
    use crate::bit_help::{X, place};

    #[test]
    fn test_place() {
        for i in 0..64 {
            assert_eq!(place(i), 2_u64.pow(i as u32));
        }
    }

    #[test]
    fn stuff() {
        let mut y = X { thing: 2};
        assert_eq!(y.thing, 2);
        *y.get_mut() |= 4;
        assert_eq!(y.thing, 6);

    }
}
