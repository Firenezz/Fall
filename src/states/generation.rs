
use bevy::prelude::*;

#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum GenerationState {
    #[default]
    Idle,
    Initializing,
    Generating,
    Done,
}

impl GenerationState {
    pub fn is_generating(&self) -> bool {
        matches!(self, Self::Generating | Self::Initializing)
    }

    pub fn is_done(&self) -> bool {
        matches!(self, Self::Done)
    }
}



