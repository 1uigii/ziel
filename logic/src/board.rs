#[derive(thiserror::Error, Debug, PartialEq, Eq, Clone, Copy)]
#[error("position already hit")]
pub struct AlreadyHitError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ShipIndexReference(u8);

impl Default for ShipIndexReference {
    fn default() -> Self {
        ShipIndexReference(u8::MAX)
    }
}

impl From<Option<u8>> for ShipIndexReference {
    fn from(index: Option<u8>) -> Self {
        ShipIndexReference(match index {
            Some(i) => i,
            None => u8::MAX,
        })
    }
}

impl From<ShipIndexReference> for Option<u8> {
    fn from(ShipIndexReference(value): ShipIndexReference) -> Self {
        match value {
            u8::MAX => None,
            n => Some(n),
        }
    }
}

impl ShipIndexReference {
    fn to_option(self) -> Option<u8> {
        self.into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttackInfo {
    Hit(bool),
    Miss,
}

#[derive(Debug, Clone)]
pub struct Board {
    ships: crate::ship::Ships,
    ship_map: [[ShipIndexReference; 10]; 10],
    hit_map: [[bool; 10]; 10],
}

impl From<crate::ship::Ships> for Board {
    fn from(ships: crate::ship::Ships) -> Self {
        let mut ship_map = [[ShipIndexReference::default(); 10]; 10];
        ships
            .into_iter()
            .enumerate()
            .flat_map(|(i, ship)| Iterator::zip(std::iter::repeat(i as u8), ship.into_iter()))
            .for_each(|(i, pos)| ship_map[pos] = ShipIndexReference::from(Some(i)));

        Board {
            ships,
            ship_map,
            hit_map: [[false; 10]; 10],
        }
    }
}

impl Board {
    pub fn from_ships(ships: crate::ship::Ships) -> Board {
        Board::from(ships)
    }

    pub fn target(&mut self, pos: crate::Position) -> Result<AttackInfo, AlreadyHitError> {
        if std::mem::replace(&mut self.hit_map[pos], true) {
            return Err(AlreadyHitError);
        }

        match self.ship_map[pos].to_option() {
            Some(i) => Ok(AttackInfo::Hit(self.is_ship_sunken(i))),
            None => Ok(AttackInfo::Miss),
        }
    }

    pub fn is_ship_sunken(&self, i: u8) -> bool {
        self.ships[i as usize].into_iter().all(|p| self.hit_map[p])
    }

    pub fn is_all_sunken(&self) -> bool {
        self.ships.into_iter().flatten().all(|p| self.hit_map[p])
    }
}
