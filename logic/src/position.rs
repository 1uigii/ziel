#[derive(thiserror::Error, Debug, PartialEq, Eq, Clone, Copy)]
#[error("position outside of bounds")]
pub struct OutOfBoundsError;

/// 2x4 byte unsigned integer vector
/// constraint to `x < 10 && y < 10`
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Position(u8);

impl TryFrom<u8> for Position {
    type Error = OutOfBoundsError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Position::try_from(<(u8, u8) as From<Position>>::from(Position(value)))
    }
}

impl TryFrom<(u8, u8)> for Position {
    type Error = OutOfBoundsError;

    fn try_from((x, y): (u8, u8)) -> Result<Self, Self::Error> {
        if x < 10 && y < 10 {
            Ok(Position((x << 4) | y))
        } else {
            Err(OutOfBoundsError)
        }
    }
}

impl From<Position> for u8 {
    fn from(Position(value): Position) -> Self {
        value
    }
}

impl From<Position> for (u8, u8) {
    fn from(Position(value): Position) -> Self {
        (value >> 4, value & 0xf)
    }
}

impl std::fmt::Debug for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (x, y) = self.to_coords();
        f.debug_struct("Position")
            .field("x", &x)
            .field("y", &y)
            .finish()
    }
}

impl Position {
    pub fn to_byte(self) -> u8 {
        self.into()
    }

    pub fn to_coords(self) -> (u8, u8) {
        self.into()
    }

    pub fn try_from_coords(
        coords: (u8, u8),
    ) -> Result<Position, <Position as TryFrom<(u8, u8)>>::Error> {
        coords.try_into()
    }

    pub fn try_from_byte(byte: u8) -> Result<Position, <Position as TryFrom<u8>>::Error> {
        byte.try_into()
    }
}

impl<T> std::ops::Index<crate::Position> for [[T; 10]; 10] {
    type Output = T;

    fn index(&self, index: crate::Position) -> &Self::Output {
        let (x, y) = index.to_coords();
        &self[y as usize][x as usize]
    }
}

impl<T> std::ops::IndexMut<crate::Position> for [[T; 10]; 10] {
    fn index_mut(&mut self, index: crate::Position) -> &mut Self::Output {
        let (x, y) = index.to_coords();
        &mut self[y as usize][x as usize]
    }
}
