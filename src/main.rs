use bevy::{math::*, prelude::*, sprite::MaterialMesh2dBundle, window::PrimaryWindow};
use std::f32::consts::PI;

const SHIP_VELOCITY: f32 = 256.;
const SHIP_ROTATION_VELOCITY: f32 = 8.;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, bevy_framepace::FramepacePlugin))
        .insert_resource(ClearColor(Color::rgb(0., 0., 0.)))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 0.6,
        })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                update_cursor_position,
                update_marker_position,
                move_ship,
                lock_camera_to_ship.after(move_ship),
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // camera
    commands.spawn((Camera2dBundle::default(), MainCamera));

    commands.init_resource::<CursorPosition>();

    // position marker
    commands.spawn((
        PositionMarker,
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0., 0.5, 0.),
                custom_size: Some(vec2(16., 16.)),
                ..default()
            },
            texture: asset_server.load("crosshair.png"),
            ..default()
        },
    ));

    // ship
    commands.spawn((
        Ship,
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::RegularPolygon::new(16., 3).into()).into(),
            material: materials.add(ColorMaterial::from(Color::rgb(0., 0.1, 0.8))),
            transform: Transform {
                scale: vec3(1., 1.5, 1.),
                ..default()
            },
            ..default()
        },
    ));

    // origin marker
    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes.add(shape::Circle::new(2.).into()).into(),
        material: materials.add(ColorMaterial::from(Color::WHITE)),
        transform: Transform::from_translation(vec3(0., 0., 999.)),
        ..default()
    });
}

/// Marks the position where the ship should go
#[derive(Component)]
struct PositionMarker;

/// Used to help identify our main camera
#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct Ship;

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Resource, Default)]
struct CursorPosition(Vec2);

fn update_cursor_position(
    mut cursor_position: ResMut<CursorPosition>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    let (camera, camera_transform) = camera_query.single();
    let window = window_query.single();

    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        cursor_position.0 = world_position;
    }
}

fn update_marker_position(
    mut marker_query: Query<&mut Transform, With<PositionMarker>>,
    cursor_position_query: Res<CursorPosition>,
    mouse_input: Res<Input<MouseButton>>,
) {
    let mut transform = marker_query.single_mut();

    for _ in mouse_input.get_pressed() {
        transform.translation = cursor_position_query.0.extend(0.);
    }
}

fn move_ship(
    mut ship_query: Query<&mut Transform, With<Ship>>,
    position_query: Query<&Transform, (With<PositionMarker>, Without<Ship>)>,
    time: Res<Time>,
) {
    let mut ship_transform = ship_query.single_mut();
    let position_marker = position_query.single();

    let direction = position_marker.translation - ship_transform.translation;
    let distance = direction.length();

    if distance >= 0.5 {
        let target_rotation = (direction.y).atan2(direction.x) - PI * 0.5;
        ship_transform.rotation = ship_transform.rotation.slerp(
            Quat::from_rotation_z(target_rotation),
            time.delta_seconds() * SHIP_ROTATION_VELOCITY,
        );

        let step_magnitude = SHIP_VELOCITY * time.delta_seconds();
        if step_magnitude > distance {
            ship_transform.translation = position_marker.translation;
        } else {
            ship_transform.translation += direction.normalize() * step_magnitude;
        }
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
        camera.translation = ship_translation;
    }
}
