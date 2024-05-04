use std::fmt;

use crate::Laast;

pub struct Similarity {
    pub edit_distance: MinMaxAverage,
}

impl fmt::Display for Similarity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Edit Distance")?;
        writeln!(f, "-------------")?;
        writeln!(f, "Min: {}", self.edit_distance.min)?;
        writeln!(f, "Max: {}", self.edit_distance.max)?;
        writeln!(f, "Average: {}", self.edit_distance.avg)?;
        Ok(())
    }
}

pub struct MinMaxAverage {
    pub min: u32,
    pub max: u32,
    pub avg: u32,
}

pub fn calculate(laasts: &[Laast]) -> Similarity {
    Similarity {
        edit_distance: MinMaxAverage {
            min: 0,
            max: 0,
            avg: 0,
        },
    }
}
