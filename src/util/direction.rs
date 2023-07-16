#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Direction{
    PosX,
    PosY,
    PosZ,
    NegX,
    NegY,
    NegZ
}

#[derive(Clone, Copy)]
pub struct DirectionIterator {
    curr: Option<Direction>
}

impl Iterator for DirectionIterator {
    type Item = Direction;
    fn next(&mut self) -> Option<Self::Item> {
        self.curr = match self.curr {
            None => Some(Direction::PosX),
            Some(Direction::PosX) => Some(Direction::PosY),
            Some(Direction::PosY) => Some(Direction::PosZ),
            Some(Direction::PosZ) => Some(Direction::NegX),
            Some(Direction::NegX) => Some(Direction::NegY),
            Some(Direction::NegY) => Some(Direction::NegZ),
            Some(Direction::NegZ) => None,
        };
        self.curr
    }
}

impl Direction {
    pub fn to_idx(self) -> usize {
        match self {
            Direction::PosX => 0,
            Direction::PosY => 1,
            Direction::PosZ => 2,
            Direction::NegX => 3,
            Direction::NegY => 4,
            Direction::NegZ => 5,
        }
    }

    pub fn opposite(self) -> Direction {
        match self {
            Direction::PosX => Direction::NegX,
            Direction::PosY => Direction::NegY,
            Direction::PosZ => Direction::NegZ,
            Direction::NegX => Direction::PosX,
            Direction::NegY => Direction::PosY,
            Direction::NegZ => Direction::PosZ,
        }
    }

    pub fn iter() -> DirectionIterator {
        DirectionIterator { curr: None }
    }
}

impl From<u64> for Direction {
    fn from(value: u64) -> Self {
        match value % 6 {
            0 => Direction::PosX,
            1 => Direction::PosY,
            2 => Direction::PosZ,
            3 => Direction::NegX,
            4 => Direction::NegY,
            5 => Direction::NegZ,
            //shouldn't happen
            _ => unreachable!()
        }
    }
}