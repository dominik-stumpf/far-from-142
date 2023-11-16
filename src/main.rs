mod debug;

use bevy::{math::*, prelude::*};
use debug::DebugPlugin;

const SHIP_VELOCITY: f32 = 48.;
const SHIP_ROTATION_VELOCITY: f32 = 6.;
const MAIN_CAMERA_TRANSFORM_OFFSET: Vec3 = Vec3::new(0., 70., -70.);

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, bevy_framepace::FramepacePlugin, DebugPlugin))
        .insert_resource(ClearColor(Color::rgb(0., 0., 0.)))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 0.4,
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
    mut materials: ResMut<Assets<StandardMaterial>>,
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
            illuminance: 1024.,
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
    commands.spawn((
        Ship,
        SceneBundle {
            scene: asset_server.load("spaceship_beta.glb#Scene0"),
            transform: Transform::from_translation(vec3(0.0, 0.0, 0.0)),
            ..default()
        },
    ));

    // origin marker
    commands.spawn((PbrBundle {
        mesh: meshes.add(Mesh::from(shape::UVSphere {
            radius: 4.,
            ..default()
        })),
        material: materials.add(Color::rgba(1., 1., 1., 0.1).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    },));
}

/// Marks the position where the ship should go
#[derive(Component)]
struct PositionMarker;

/// Used to help identify our main camera
#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct Ship;

#[derive(Resource, Default)]
struct CursorPosition(Vec3);

fn update_cursor_position(
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    windows: Query<&Window>,
    mut gizmos: Gizmos,
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

    gizmos.circle(point + plane.up() * 0.01, plane.up(), 0.8, Color::WHITE);
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
    mut ship_query: Query<&mut Transform, With<Ship>>,
    position_query: Query<&Transform, (With<PositionMarker>, Without<Ship>)>,
    time: Res<Time>,
) {
    let mut ship_transform = ship_query.single_mut();
    let position_marker = position_query.single();

    let direction = position_marker.translation - ship_transform.translation;
    let distance = direction.length();

    if distance >= 0.5 {
        let target_rotation = (direction.x).atan2(direction.z);
        ship_transform.rotation = ship_transform.rotation.slerp(
            Quat::from_rotation_y(target_rotation),
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
        camera.translation =
            Transform::from_translation(ship_translation + MAIN_CAMERA_TRANSFORM_OFFSET)
                .looking_at(ship_translation, Vec3::Z)
                .translation;
    }
}
