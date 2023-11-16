use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

pub struct DebugPlugin;

/// A unit struct to help identify the FPS UI component
#[derive(Component)]
struct FpsText;

/// A resouce used for determining the debug UI update frequency
#[derive(Resource)]
struct DebugUpdateTimer(Timer);

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .insert_resource(DebugUpdateTimer(Timer::from_seconds(
                0.5,
                TimerMode::Repeating,
            )))
            .add_systems(Update, text_update_system)
            .add_plugins(FrameTimeDiagnosticsPlugin);
    }
}
fn setup(mut commands: Commands) {
    // fps debug
    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "FPS: ",
                TextStyle {
                    font_size: 10.0,
                    ..default()
                },
            ),
            TextSection::from_style(TextStyle {
                font_size: 10.0,
                ..default()
            }),
        ]),
        FpsText,
    ));
}

fn text_update_system(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut Text, With<FpsText>>,
    time: Res<Time>,
    mut timer: ResMut<DebugUpdateTimer>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        for mut text in &mut query {
            if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
                if let Some(value) = fps.average() {
                    text.sections[1].value = format!("{value:.2}");
                    let color;
                    if value < 30. {
                        color = Color::RED;
                    } else if value < 60. {
                        color = Color::GOLD;
                    } else {
                        color = Color::GREEN;
                    }

                    text.sections[1].style.color = color;
                }
            }
        }
    }
}
