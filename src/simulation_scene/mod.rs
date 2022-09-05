mod loaded_scene;
mod scene_data;
mod systems;

pub use loaded_scene::LoadedScene;
pub use scene_data::{Empty, SceneData, SimulationScene};

use bevy::{
    app::{App, Plugin},
    ecs::schedule::ParallelSystemDescriptorCoercion,
};

pub type SceneCollection = Vec<SimulationScene>;

pub trait AddScene {
    fn with<S>(self, scene: S) -> Self
    where
        S: SceneData + Send + Sync + 'static;

    fn add<S>(&mut self, scene: S)
    where
        S: SceneData + Send + Sync + 'static;
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
}

pub struct SimulationScenePlugin;

impl Plugin for SimulationScenePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LoadedScene::new(Empty {})).add_system(
            systems::scene_cleanup_and_reload.with_run_criteria(systems::scene_changed),
        );
    }
}
