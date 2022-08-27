mod input;
mod nbody;
mod trails;

use std::f32::INFINITY;
use std::time::Duration;

use bevy::input::mouse::MouseButtonInput;
use bevy::input::ButtonState;
use bevy_prototype_debug_lines::DebugLines;
use nbody::{ParticularPlugin, PointMass};
use trails::{DrawTrail, TrailsPlugin};

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::{prelude::*, window::PresentMode};
use bevy_inspector_egui::prelude::*;
use bevy_mouse_tracking_plugin::{MousePosPlugin, MousePosWorld};
use bevy_pancam::{PanCam, PanCamPlugin};
use bevy_prototype_lyon::entity::ShapeBundle;
use bevy_prototype_lyon::prelude::*;
use bevy_prototype_lyon::shapes::Circle;
use heron::{prelude::*, PhysicsSteps};
use rand::{thread_rng, Rng};

const G: f32 = 1000.0;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "I am a window!".to_string(),
            // width: 1500.0,
            // height: 900.0,
            present_mode: PresentMode::AutoNoVsync,
            ..default()
        })
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(PhysicsSteps::from_steps_per_seconds(60.0))
        .add_plugin(LogDiagnosticsPlugin {
            wait_duration: Duration::from_secs_f32(1.0),
            ..default()
        })
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        // .add_plugin(WorldInspectorPlugin::new())
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .add_plugin(PanCamPlugin::default())
        .add_plugin(MousePosPlugin::SingleCamera)
        .add_plugin(PhysicsPlugin::default())
        .add_plugin(TrailsPlugin)
        .add_plugin(ParticularPlugin)
        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_random_bodies)
        .add_system(place_body)
        .add_system(pause_sim)
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands
        .spawn_bundle(Camera2dBundle::default())
        .insert(PanCam {
            grab_buttons: vec![MouseButton::Right, MouseButton::Middle],
            ..default()
        });
}

fn spawn_random_bodies(mut commands: Commands) {
    let small_mass = 0.0001;
    let big_mass = 10000.0;

    let mut rng = thread_rng();

    commands.spawn_bundle(BodyBundle::new(
        Vec3::ZERO,
        Velocity::from_linear(Vec3::ZERO),
        10.0,
        INFINITY,
        PointMass::HasGravity { mass: big_mass },
    ));

    for _ in 0..100 {
        let range = 1000.0;
        let pos = Vec3::new(
            rng.gen_range(-range..range),
            rng.gen_range(-range..range),
            0.0,
        );

        let _direction = pos - Vec3::ZERO;
        let direction = Vec2::new(_direction.x, _direction.y);
        let distance = direction.length_squared();

        let vel = (G * (big_mass + small_mass)).sqrt() * distance.powf(-0.75);

        let velvec = Vec3::new(-direction.y * vel, direction.x * vel, 0.0);

        commands
            .spawn_bundle(BodyBundle::new(
                pos,
                Velocity::from_linear(velvec),
                2.0,
                1.0,
                PointMass::HasGravity { mass: small_mass },
            ))
            // .insert(DrawTrail::new(20.0, 1))
            ;
    }
}

fn place_body(
    mut commands: Commands,
    mouse_pos: Res<MousePosWorld>,
    mut click_event: EventReader<MouseButtonInput>,
    mut place_pos: Local<Option<Vec3>>,
    mut lines: ResMut<DebugLines>,
) {
    let mouse_pos = mouse_pos.truncate().extend(0.0);

    for event in click_event.iter() {
        if event.button == MouseButton::Left {
            match event.state {
                ButtonState::Pressed => *place_pos = Some(mouse_pos),
                ButtonState::Released => {
                    if let Some(place_pos) = place_pos.take() {
                        commands
                            .spawn_bundle(BodyBundle::new(
                                place_pos,
                                Velocity::from_linear(place_pos - mouse_pos),
                                2.0,
                                1.0,
                                PointMass::HasGravity { mass: 0.1 },
                            ))
                            // .insert(DrawTrail::new(20.0, 1))
                            ;
                    }
                }
            }
        }
    }

    if let Some(place_pos) = *place_pos {
        let scale = (mouse_pos.distance_squared(place_pos).powf(0.04) - 1.0).clamp(0.0, 1.0);
        lines.line_colored(
            place_pos,
            mouse_pos,
            0.0,
            Color::rgb(scale, 1.0 - scale, 0.0),
        )
    }
}

fn pause_sim(keys: Res<Input<KeyCode>>, mut time: ResMut<PhysicsTime>) {
    let is_paused = time.scale() == 0.0;
    if keys.just_pressed(KeyCode::Space) {
        match is_paused {
            true => time.resume(),
            false => time.pause(),
        }
    }
}

#[derive(Bundle)]
struct BodyBundle {
    #[bundle]
    shape_bundle: ShapeBundle,
    collider: CollisionShape,
    material: PhysicMaterial,
    rigidbody: RigidBody,
    velocity: Velocity,
    acceleration: Acceleration,
    point_mass: PointMass,
}

impl BodyBundle {
    fn new(
        position: Vec3,
        velocity: Velocity,
        radius: f32,
        density: f32,
        point_mass: PointMass,
    ) -> Self {
        Self {
            shape_bundle: GeometryBuilder::build_as(
                &Circle {
                    radius,
                    center: Vec2::ZERO,
                },
                DrawMode::Fill(FillMode::color(Color::WHITE)),
                Transform::from_translation(position),
            ),
            collider: CollisionShape::Sphere { radius },
            material: PhysicMaterial {
                restitution: 0.0,
                density,
                friction: 0.0,
            },
            rigidbody: RigidBody::Dynamic,
            velocity,
            acceleration: Acceleration::default(),
            point_mass,
        }
    }
}
