pub mod board;
pub mod position;
pub mod ship;

pub use board::Board;
pub use position::Position;
pub use ship::Ships;

#[derive(thiserror::Error, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Error {
    #[error("logic :: position :: {0}")]
    PositionOutOfBounds(#[from] position::OutOfBoundsError),
    #[error("logic :: ship :: {0}")]
    ShipOutOfBounds(#[from] ship::OutOfBoundsError),
    #[error("logic :: ship :: {0}")]
    ShipCollection(#[from] ship::ShipCollectionError),
    #[error("logic :: board :: {0}")]
    Board(#[from] board::AlreadyHitError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_translations() {
        let coords = (1, 2);
        let position = position::Position::try_from_coords(coords).unwrap();
        assert_eq!(position.to_coords(), coords);

        let byte = position.to_byte();
        let position = position::Position::try_from_byte(byte).unwrap();
        assert_eq!(position.to_byte(), byte);
    }

    #[test]
    fn invalid_positions() {
        let coords = (10, 2);
        let position = position::Position::try_from_coords(coords);
        assert_eq!(position, Err(position::OutOfBoundsError));

        let byte = 255;
        let position = position::Position::try_from_byte(byte);
        assert_eq!(position, Err(position::OutOfBoundsError));
    }

    #[test]
    fn ship_area_iterator() {
        let ship_plan = ship::ShipPlan::Horizontal {
            pos: position::Position::try_from_coords((3, 2)).unwrap(),
            length: 4,
        };
        let ship = ship::Ship::try_from(ship_plan).unwrap();

        let mut positions: Vec<_> = ship.into_iter().collect();
        positions.sort_by_key(|p| p.to_byte());
        let mut expected = vec![
            position::Position::try_from_coords((3, 2)).unwrap(),
            position::Position::try_from_coords((4, 2)).unwrap(),
            position::Position::try_from_coords((5, 2)).unwrap(),
            position::Position::try_from_coords((6, 2)).unwrap(),
        ];
        expected.sort_by_key(|p| p.to_byte());
        assert_eq!(positions, expected);

        let ship_plan = ship::ShipPlan::Vertical {
            pos: position::Position::try_from_coords((3, 2)).unwrap(),
            length: 4,
        };
        let ship = ship::Ship::try_from(ship_plan).unwrap();

        let mut positions: Vec<_> = ship.into_iter().collect();
        positions.sort_by_key(|p| p.to_byte());
        let mut expected = vec![
            position::Position::try_from_coords((3, 2)).unwrap(),
            position::Position::try_from_coords((3, 3)).unwrap(),
            position::Position::try_from_coords((3, 4)).unwrap(),
            position::Position::try_from_coords((3, 5)).unwrap(),
        ];
        expected.sort_by_key(|p| p.to_byte());
        assert_eq!(positions, expected);
    }

    #[test]
    fn ship5_to_ships() {
        let ships = [
            ship::Ship::try_from(ship::ShipPlan::Horizontal {
                pos: position::Position::try_from_coords((0, 1)).unwrap(),
                length: 5,
            })
            .unwrap(),
            ship::Ship::try_from(ship::ShipPlan::Horizontal {
                pos: position::Position::try_from_coords((7, 2)).unwrap(),
                length: 2,
            })
            .unwrap(),
            ship::Ship::try_from(ship::ShipPlan::Horizontal {
                pos: position::Position::try_from_coords((1, 6)).unwrap(),
                length: 3,
            })
            .unwrap(),
            ship::Ship::try_from(ship::ShipPlan::Vertical {
                pos: position::Position::try_from_coords((5, 4)).unwrap(),
                length: 4,
            })
            .unwrap(),
            ship::Ship::try_from(ship::ShipPlan::Vertical {
                pos: position::Position::try_from_coords((7, 7)).unwrap(),
                length: 3,
            })
            .unwrap(),
        ];
        let ships = ship::Ships::try_from(ships).unwrap();
    }

    #[test]
    fn ship5_to_ships_fail() {
        let ships = [
            ship::Ship::try_from(ship::ShipPlan::Horizontal {
                pos: position::Position::try_from_coords((0, 1)).unwrap(),
                length: 3,
            })
            .unwrap(),
            ship::Ship::try_from(ship::ShipPlan::Horizontal {
                pos: position::Position::try_from_coords((7, 2)).unwrap(),
                length: 2,
            })
            .unwrap(),
            ship::Ship::try_from(ship::ShipPlan::Horizontal {
                pos: position::Position::try_from_coords((1, 6)).unwrap(),
                length: 3,
            })
            .unwrap(),
            ship::Ship::try_from(ship::ShipPlan::Vertical {
                pos: position::Position::try_from_coords((5, 4)).unwrap(),
                length: 4,
            })
            .unwrap(),
            ship::Ship::try_from(ship::ShipPlan::Vertical {
                pos: position::Position::try_from_coords((7, 7)).unwrap(),
                length: 3,
            })
            .unwrap(),
        ];
        assert_eq!(
            Err(ship::ShipCollectionError::InvalidShipLengths),
            ship::Ships::try_from(ships)
        );

        let ships = [
            ship::Ship::try_from(ship::ShipPlan::Horizontal {
                pos: position::Position::try_from_coords((0, 1)).unwrap(),
                length: 5,
            })
            .unwrap(),
            ship::Ship::try_from(ship::ShipPlan::Horizontal {
                pos: position::Position::try_from_coords((7, 2)).unwrap(),
                length: 2,
            })
            .unwrap(),
            ship::Ship::try_from(ship::ShipPlan::Horizontal {
                pos: position::Position::try_from_coords((4, 6)).unwrap(),
                length: 3,
            })
            .unwrap(),
            ship::Ship::try_from(ship::ShipPlan::Vertical {
                pos: position::Position::try_from_coords((5, 4)).unwrap(),
                length: 4,
            })
            .unwrap(),
            ship::Ship::try_from(ship::ShipPlan::Vertical {
                pos: position::Position::try_from_coords((7, 7)).unwrap(),
                length: 3,
            })
            .unwrap(),
        ];
        assert_eq!(
            Err(ship::ShipCollectionError::Overlap),
            ship::Ships::try_from(ships)
        );
    }
}
