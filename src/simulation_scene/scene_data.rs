use std::fmt::Display;

use bevy::ecs::system::EntityCommands;
use bevy_egui::egui::Ui;

pub type SimulationScene = Box<dyn SceneData + Send + Sync>;

pub trait SceneDataClone {
    fn clone_box(&self) -> SimulationScene;
}

impl<T: 'static + SceneData + Send + Sync + Clone> SceneDataClone for T {
    fn clone_box(&self) -> SimulationScene {
        Box::new(self.clone())
    }
}

pub trait SceneData: SceneDataClone + Display {
    fn instance(&self, scene_commands: EntityCommands);

    fn show_ui(&mut self, ui: &mut Ui);

    fn max_spawnable_mass(&self) -> f32;
}

impl Clone for SimulationScene {
    fn clone(&self) -> SimulationScene {
        self.clone_box()
    }
}

impl PartialEq for SimulationScene {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}

#[derive(Clone, Default)]
pub struct Empty;

impl Display for Empty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Empty")
    }
}

impl SceneData for Empty {
    fn instance(&self, _: EntityCommands) {}

    fn show_ui(&mut self, _: &mut Ui) {}

    fn max_spawnable_mass(&self) -> f32 {
        100.0
    }
}
