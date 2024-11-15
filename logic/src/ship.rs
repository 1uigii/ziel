#[derive(thiserror::Error, Debug, PartialEq, Eq, Clone, Copy)]
#[error("ship body is out of bounds")]
pub struct OutOfBoundsError;

/// Error that signals if any step in the ship collection process has gone wrong.
///
/// The collection function shortcircuits,
/// so not every error will be present, only the first found
#[derive(thiserror::Error, Debug, PartialEq, Eq, Clone, Copy)]
pub enum ShipCollectionError {
    #[error("ship overlaps with another ship")]
    Overlap,
    #[error("not all needed ship lengths found")]
    InvalidShipLengths,
}

/// Plan how a ship could be positioned.
/// May be an invalid position
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShipPlan {
    Horizontal { pos: crate::Position, length: u8 },
    Vertical { pos: crate::Position, length: u8 },
}

/// A ship that is definetly within the bounds of the game board
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ship(ShipPlan);

impl TryFrom<ShipPlan> for Ship {
    type Error = OutOfBoundsError;

    fn try_from(ship: ShipPlan) -> Result<Self, Self::Error> {
        if match ship {
            ShipPlan::Horizontal { pos, length } => pos.to_coords().0 + length,
            ShipPlan::Vertical { pos, length } => pos.to_coords().1 + length,
        } <= 10
        {
            Ok(Ship(ship))
        } else {
            Err(OutOfBoundsError)
        }
    }
}

impl From<Ship> for ShipPlan {
    fn from(Ship(ship): Ship) -> Self {
        ship
    }
}

impl Ship {
    pub fn to_ship_plan(self) -> ShipPlan {
        self.into()
    }
}

/// Iterates over every position the ship occupies
/// These positions may not be ascending in order
pub struct ShipAreaIterator(ShipPlan);

impl IntoIterator for Ship {
    type Item = crate::Position;

    type IntoIter = ShipAreaIterator;

    fn into_iter(self) -> Self::IntoIter {
        ShipAreaIterator(self.into())
    }
}

impl Iterator for ShipAreaIterator {
    type Item = crate::Position;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0 {
            ShipPlan::Horizontal { pos, length } => {
                let (x, y) = pos.to_coords();
                let length = length.checked_sub(1)?;
                self.0 = ShipPlan::Horizontal { pos, length };
                Some(
                    crate::Position::try_from_coords((x + length, y))
                        .expect("ship is already checked for bounds"),
                )
            }
            ShipPlan::Vertical { pos, length } => {
                let (x, y) = pos.to_coords();
                let length = length.checked_sub(1)?;
                self.0 = ShipPlan::Vertical { pos, length };
                Some(
                    crate::Position::try_from_coords((x, y + length))
                        .expect("ship is already checked for bounds"),
                )
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let length = match self.0 {
            ShipPlan::Horizontal { length, .. } => length as usize,
            ShipPlan::Vertical { length, .. } => length as usize,
        } as usize;
        (length, Some(length))
    }
}

/// A collection of five ships that are definetly valid and not overlapping
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ships([Ship; 5]);

impl TryFrom<[Ship; 5]> for Ships {
    type Error = ShipCollectionError;

    fn try_from(ships: [Ship; 5]) -> Result<Self, Self::Error> {
        const SHIP_LENGTHS: [u8; 5] = [2, 3, 3, 4, 5];

        let mut ship_map = [[false; 10]; 10];
        let mut ship_length_map = [false; 5];
        for ship in ships {
            let ship_length = match ship.to_ship_plan() {
                ShipPlan::Horizontal { length, .. } => length,
                ShipPlan::Vertical { length, .. } => length,
            };
            *Iterator::zip(ship_length_map.iter_mut(), SHIP_LENGTHS.into_iter())
                .find_map(|(found, length)| {
                    if !*found && length == ship_length {
                        Some(found)
                    } else {
                        None
                    }
                })
                .ok_or(ShipCollectionError::InvalidShipLengths)? = true;

            for pos in ship {
                if std::mem::replace(&mut ship_map[pos], true) {
                    return Err(ShipCollectionError::Overlap);
                }
            }
        }

        Ok(Ships(ships))
    }
}

impl Ships {
    pub fn into_ship_array(self) -> [Ship; 5] {
        self.0
    }
}

impl IntoIterator for Ships {
    type Item = Ship;

    type IntoIter = std::array::IntoIter<Self::Item, 5>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'s> IntoIterator for &'s Ships {
    type Item = &'s Ship;

    type IntoIter = std::slice::Iter<'s, Ship>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl std::ops::Index<usize> for Ships {
    type Output = Ship;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}
