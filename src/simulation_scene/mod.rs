mod loaded_scene;
mod scene_data;
mod spawnable;
mod systems;

pub use loaded_scene::LoadedScene;
pub use scene_data::{Empty, SceneData, SimulationScene};
pub use spawnable::Spawnable;

use bevy::app::{App, CoreStage, Plugin};

pub type SceneCollection = Vec<SimulationScene>;

pub trait AddScene {
    fn with<S>(self, scene: S) -> Self
    where
        S: SceneData + Send + Sync + 'static;

    fn add<S>(&mut self, scene: S)
    where
        S: SceneData + Send + Sync + 'static;

    fn with_scene<S>(self) -> Self
    where
        S: SceneData + Default + Send + Sync + 'static;
}

impl AddScene for SceneCollection {
    fn with<S>(mut self, scene: S) -> Self
    where
        S: SceneData + Send + Sync + 'static,
    {
        self.push(Box::new(scene));
        self
    }

    fn add<S>(&mut self, scene: S)
    where
        S: SceneData + Send + Sync + 'static,
    {
        self.push(Box::new(scene));
    }

    fn with_scene<S>(mut self) -> Self
    where
        S: SceneData + Default + Send + Sync + 'static,
    {
        self.push(Box::new(S::default()));
        self
    }
}

pub struct SimulationScenePlugin;

impl Plugin for SimulationScenePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LoadedScene::new(Empty {}))
            .add_system_to_stage(CoreStage::PreUpdate, systems::scene_cleanup_and_reload)
            .add_system(systems::show_ui);
    }
}
