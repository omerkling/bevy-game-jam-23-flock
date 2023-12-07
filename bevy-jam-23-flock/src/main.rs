use bevy::{prelude::*, render::camera::ScalingMode, window::PrimaryWindow};

#[derive(Component)]
struct Player{
    velocity: Vec2,
}

#[derive(Component)]
struct Bird {
    velocity: Vec2,
}

#[derive(Component)]
struct MainCamera;

fn main() {
    App::new()
    .add_plugins(DefaultPlugins)   
    .add_systems(Startup, (setup, spawn_player, spawn_birds))
    .add_systems(Update, update_player)
    .add_systems(Update, update_birds
        .after(update_player)
    )
    .insert_resource(ClearColor(Color::rgb(0.53, 0.53, 0.53)))
    .run();
}

fn setup(mut commands: Commands) {
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection.scaling_mode = ScalingMode::FixedVertical(100.);
    commands.spawn((camera_bundle, MainCamera));
}

fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Player {velocity: Vec2 {x: 0., y: 0.}},
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0., 0.47, 1.),
                custom_size: Some(Vec2::new(1., 1.)),
                ..default()
            },
            ..default()}
    ));
}

fn spawn_birds(mut commands: Commands) {
    for n in 0..10 {
        commands.spawn((
            Bird {velocity: Vec2::new(n as f32, n as f32)},
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.1 * n as f32, 0., 1.),
                    custom_size: Some(Vec2::new(1., 1.)),
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(n as f32, n as f32, 0.)),
                ..default()}
        ));
    }
}

fn update_player(
    mut players: Query<(&mut Transform, &mut Player), With<Player>>,
    // query to get the window (so we can read the current cursor position)
    q_window: Query<&Window, With<PrimaryWindow>>,
    // query to get camera transform
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so Query::single() is OK
    let (camera, camera_transform) = q_camera.single();

    // There is only one primary window, so we can similarly get it from the query:
    let window = q_window.single();

    // check if the cursor is inside the window and get its position
    // then, ask bevy to convert into world coordinates, and truncate to discard Z
    if let Some(world_position) = window.cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin)
    {        
       for (mut transform, mut player) in &mut players {
        let old_pos = transform.translation;
        transform.translation = Vec3::new(world_position.x, world_position.y, 0.);
        player.velocity = (transform.translation - old_pos).xy();
       }
    }
}

fn update_birds(    
    time: Res<Time>,
    players: Query<&Transform, (With<Player>, Without<Bird>)>,
    mut birds: Query<(&mut Transform, &mut Bird), With<Bird>>,
) {
    let delta = time.delta_seconds();
    let player = players.single();
    for (mut transform, mut bird) in &mut birds {
        let to_player = player.translation - transform.translation;       
        bird.velocity =  to_player.normalize_or_zero().xy();
        transform.translation = Vec3::new(transform.translation.x + bird.velocity.x, transform.translation.y + bird.velocity.y, 0.);
        // eprintln!("Bird v {} and p {}", bird.velocity, transform.translation);
    }
}