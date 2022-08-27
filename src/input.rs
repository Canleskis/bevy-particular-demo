use bevy::{input::Input, prelude::*};

pub enum Wins {
    Negative,
    Positive,
    Neither,
}

pub trait CompositeInput {
    fn read(&self, negative: KeyCode, positive: KeyCode, dominant: Wins) -> f32;
}

impl CompositeInput for Input<KeyCode> {
    fn read(&self, negative: KeyCode, positive: KeyCode, dominant: Wins) -> f32 {
        match (self.pressed(negative), self.pressed(positive), dominant) {
            (true, false, _) => -1.0,
            (false, true, _) => 1.0,
            (true, true, Wins::Negative) => -1.0,
            (true, true, Wins::Positive) => 1.0,
            _ => 0.0,
        }
    }
}
