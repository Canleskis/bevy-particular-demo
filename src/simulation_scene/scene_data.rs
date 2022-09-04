use bevy::ecs::system::EntityCommands;
use bevy_egui::egui;

pub type SimulationScene = Box<dyn SceneData + Send + Sync>;

pub trait SceneDataClone {
    fn clone_box(&self) -> SimulationScene;
}

impl<T: 'static + SceneData + Send + Sync + Clone> SceneDataClone for T {
    fn clone_box(&self) -> SimulationScene {
        Box::new(self.clone())
    }
}

pub trait SceneData: SceneDataClone + std::fmt::Display {
    fn instance(&self, scene_commands: EntityCommands);

    fn show_ui(&mut self, ui: &mut egui::Ui);

    fn max_spawnable_mass(&self) -> f32;
}

impl Clone for SimulationScene {
    fn clone(&self) -> SimulationScene {
        self.clone_box()
    }
}
