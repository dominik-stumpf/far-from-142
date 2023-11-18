mod debug;

use std::f32::consts::PI;

use bevy::{math::*, prelude::*};
use bevy_hanabi::prelude::*;
use debug::DebugPlugin;

const SHIP_VELOCITY: f32 = 48.;
const SHIP_ROTATION_VELOCITY: f32 = 6.;
const SHIP_MAX_TILT_ANGLE: f32 = PI * 0.16;
// const SHIP_TILT_VELOCITY: f32 = 2.;
const MAIN_CAMERA_TRANSFORM_OFFSET: Vec3 = Vec3::new(0., 70., -70.);

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            bevy_framepace::FramepacePlugin,
            DebugPlugin,
            HanabiPlugin,
        ))
        .insert_resource(Msaa::default())
        .insert_resource(ClearColor(Color::rgb(0., 0., 0.)))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 0.2,
        })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                update_cursor_position,
                update_marker_position,
                move_ship,
                lock_camera_to_ship.after(move_ship),
                draw_gizmos,
                emit,
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut effects: ResMut<Assets<EffectAsset>>,
    asset_server: Res<AssetServer>,
) {
    // camera
    commands.spawn((
        MainCamera,
        Camera3dBundle {
            transform: Transform::from_translation(MAIN_CAMERA_TRANSFORM_OFFSET)
                .looking_at(Vec3::ZERO, Vec3::Z),
            camera: Camera { ..default() },
            ..default()
        },
    ));

    // scene lighting
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::WHITE,
            illuminance: 16384.,
            ..default()
        },
        transform: Transform {
            rotation: Quat::from_scaled_axis(vec3(0., 1., 0.8)),
            ..default()
        },
        ..default()
    });

    commands.init_resource::<CursorPosition>();

    // position marker
    commands.spawn((
        PositionMarker,
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube {
                size: 2.,
                ..default()
            })),
            material: materials.add(Color::rgba(0., 0.5, 1., 0.1).into()),
            // transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        },
    ));

    // ship
    let mut ship = commands.spawn(ShipBundle {
        ship: Ship::default(),
        scene_bundle: SceneBundle {
            scene: asset_server.load("spaceship_beta.glb#Scene0"),
            transform: Transform::from_translation(vec3(0.0, 0.0, 0.0)),
            ..default()
        },
    });

    // setup particle emitter
    let mut gradient = Gradient::new();
    gradient.add_key(0.0, Vec4::new(0.1, 0.1, 1.0, 1.0));
    gradient.add_key(1.0, Vec4::new(0.4, 0.4, 0.6, 0.0));

    let spawner = Spawner::rate(512.0.into()).with_starts_active(true);

    let writer = ExprWriter::new();

    let age = writer.lit(0.).expr();
    let init_age = SetAttributeModifier::new(Attribute::AGE, age);

    let lifetime = writer.lit(0.1).uniform(writer.lit(0.4)).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    let init_pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(0.4).expr(),
        dimension: ShapeDimension::Volume,
    };
    // let init_pos = SetPositionCone3dModifier {
    //     base_radius: writer.lit(0.).expr(),
    //     top_radius: writer.lit(0.8).expr(),
    //     height: writer.lit(8.).expr(),
    //     dimension: ShapeDimension::Volume,
    // };

    let init_vel = SetVelocitySphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        speed: writer.lit(0.0).expr(),
    };

    let effect = effects.add(
        EffectAsset::new(32768, spawner, writer.finish())
            .with_name("activate")
            .init(init_pos)
            .init(init_vel)
            .init(init_age)
            .init(init_lifetime)
            .render(SizeOverLifetimeModifier {
                gradient: Gradient::constant(Vec2::splat(0.4)),
                screen_space_size: false,
            })
            .render(ColorOverLifetimeModifier { gradient }),
    );

    // commands.spawn((
    //     Name::new("trail"),
    //     ParticleEffectBundle {
    //         effect: ParticleEffect::new(effect),
    //         transform: Transform::IDENTITY,
    //         ..Default::default()
    //     },
    // ));

    ship.with_children(|node| {
        node.spawn(
            ParticleEffectBundle {
                effect: ParticleEffect::new(effect),
                transform: Transform::from_xyz(0., 0., -4.),
                ..default()
            }
            .with_spawner(spawner),
        )
        .insert(Name::new("effect"));
    });
}

/// Marks the position where the ship should go
#[derive(Component)]
struct PositionMarker;

/// Used to help identify our main camera
#[derive(Component)]
struct MainCamera;

#[derive(Component, Default, Debug)]
struct Ship {
    previous_rotation_angle: f32,
}

#[derive(Bundle)]
struct ShipBundle {
    ship: Ship,
    scene_bundle: SceneBundle,
}

#[derive(Resource, Default)]
struct CursorPosition(Vec3);

fn update_cursor_position(
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    windows: Query<&Window>,
    mut cursor_position_resource: ResMut<CursorPosition>,
) {
    let (camera, camera_transform) = camera_query.single();
    let plane = Transform::from_xyz(0., 0., 0.);

    let Some(cursor_position) = windows.single().cursor_position() else {
        return;
    };

    let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    let Some(distance) = ray.intersect_plane(plane.translation, plane.up()) else {
        return;
    };
    let point = ray.get_point(distance);

    cursor_position_resource.0 = point + plane.up() * 0.01;
}

fn update_marker_position(
    mut marker_query: Query<&mut Transform, With<PositionMarker>>,
    cursor_position_query: Res<CursorPosition>,
    mouse_input: Res<Input<MouseButton>>,
) {
    let mut transform = marker_query.single_mut();

    if mouse_input.pressed(MouseButton::Left) {
        transform.translation = cursor_position_query.0;
    }
}

fn move_ship(
    mut ship_query: Query<(&mut Transform, &mut Ship)>,
    position_query: Query<&Transform, (With<PositionMarker>, Without<Ship>)>,
    time: Res<Time>,
) {
    let (mut ship_transform, mut ship) = ship_query.single_mut();
    let position_marker = position_query.single();

    // compare between only x and z
    let direction = position_marker.translation - ship_transform.translation;
    let distance = direction.length_squared();

    if distance >= 0.5 {
        let target_rotation = (direction.x).atan2(direction.z);

        let positive_new_tilt_angle = angle_to_positive_domain(target_rotation);
        let positive_previous_tilt_angle = angle_to_positive_domain(ship.previous_rotation_angle);
        let tilt_difference = positive_new_tilt_angle - positive_previous_tilt_angle;

        // println!("{:?}", tilt_difference);

        ship_transform.rotation = ship_transform.rotation.slerp(
            Quat::from_euler(
                EulerRot::XYZ,
                0.,
                target_rotation,
                (tilt_difference * -6.).clamp(-SHIP_MAX_TILT_ANGLE, SHIP_MAX_TILT_ANGLE),
            ),
            time.delta_seconds() * SHIP_ROTATION_VELOCITY,
        );

        ship.previous_rotation_angle = target_rotation;

        let step_magnitude = SHIP_VELOCITY * time.delta_seconds();
        if step_magnitude.powi(2) > distance {
            ship_transform.translation = position_marker.translation;
        } else {
            ship_transform.translation += direction.normalize() * step_magnitude;
        }
    }
}

fn angle_to_positive_domain(angle: f32) -> f32 {
    if angle >= 0. {
        angle
    } else {
        2. * PI - angle.abs()
    }
}

fn lock_camera_to_ship(
    mut param_query: ParamSet<(
        Query<&Transform, With<Ship>>,
        Query<&mut Transform, With<MainCamera>>,
    )>,
) {
    let mut ship_translation = Vec3::ZERO;
    for ship in param_query.p0().iter_mut() {
        ship_translation = ship.translation;
    }

    for mut camera in param_query.p1().iter_mut() {
        camera.translation =
            Transform::from_translation(ship_translation + MAIN_CAMERA_TRANSFORM_OFFSET)
                .looking_at(ship_translation, Vec3::Z)
                .translation;
    }
}

fn draw_gizmos(mut gizmos: Gizmos) {
    gizmos.circle(Vec3::ZERO, Vec3::Y, 4., Color::WHITE);
}

fn emit(
    // mut ship_query: Query<(&mut Ship, &mut Transform, &Children)>,
    mut spawner_query: Query<(&mut Transform, &mut ParticleEffect)>,
    time: Res<Time>,
) {
    // let (mut ship, mut transform, children) = ship_query.single_mut();
    let (mut transform, mut effect) = spawner_query.single_mut();
    // transform.rotation = Quat::IDENTITY;
    // transform.translation.x += 8. * time.delta_seconds();
    // transform.translation.y = transform.translation.x.sin();
    // transform.translation = Vec3::splat(time.elapsed_seconds().sin() * 16.);

    // spawner.set_active(true);
}
