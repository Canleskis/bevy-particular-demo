use crate::{SceneData, SimulationScene};
use bevy::ecs::{entity::Entity, system::EntityCommands};

pub struct LoadedScene {
    scene: SimulationScene,
    entity: Option<Entity>,
}

impl LoadedScene {
    pub fn new<S>(scene: S) -> Self
    where
        S: SceneData + Send + Sync + 'static,
    {
        Self {
            scene: Box::new(scene),
            entity: None,
        }
    }

    pub fn load(&mut self, scene: SimulationScene) {
        self.scene = scene;
    }

    pub fn loaded(&mut self) -> &SimulationScene {
        &self.scene
    }

    pub fn spawned(&mut self, entity: Entity) {
        self.entity.get_or_insert(entity);
    }

    pub fn despawned(&mut self) {
        self.entity = None;
    }

    pub fn entity(&self) -> Entity {
        self.entity
            .unwrap_or_else(|| panic!("No entity for {}", self.scene))
    }

    pub fn get_entity(&self) -> Option<Entity> {
        self.entity
    }

    pub fn instance(&self, scene_commands: EntityCommands) {
        self.scene.instance(scene_commands)
    }

    pub fn max_spawnable_mass(&self) -> f32 {
        self.scene.max_spawnable_mass()
    }
}
