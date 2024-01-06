use bevy::{prelude::*, render::camera::ScalingMode, window::PrimaryWindow};
use kiddo::{float::kdtree::KdTree, SquaredEuclidean};
use rustc_hash::FxHashMap;

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

#[derive(Resource)]
struct Birds {
    count: usize,
}

fn main() {
    fastrand::seed(37);
    App::new()
    .add_plugins(DefaultPlugins)   
    .add_systems(Startup, (setup, spawn_player, spawn_birds))
    .add_systems(Update, update_player)
    .add_systems(Update, update_birds
        .after(update_player)
    )
    .insert_resource(ClearColor(Color::rgb(1., 1., 1.) * 0.3))
    .run();
}

fn setup(mut commands: Commands) {
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection.scaling_mode = ScalingMode::FixedVertical(200.);
    commands.spawn((camera_bundle, MainCamera));
}

fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Player {velocity: Vec2 {x: 0., y: 0.}},
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0., 1., 0.5),
                custom_size: Some(Vec2::new(1., 1.)),
                ..default()
            },
            ..default()}
    ));
}

fn spawn_birds(mut commands: Commands) {
    let c: usize = 1000;
    for n in 0..c {
        commands.spawn((
            Bird {velocity: Vec2::new(0 as f32, 0 as f32)},
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb((1./c as f32) * n as f32, 0., 1.),
                    custom_size: Some(Vec2::new(1., 1.)),
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new((fastrand::f32() - 0.5) * 100., (fastrand::f32() - 0.5) * 100., 0.)),
                ..default()}
        ));
    }
    commands.insert_resource(Birds{count: c});
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

struct BirdData {
    translation: Vec2,
    velocity: Vec2,
}

fn strength(min: f32, max:f32, d: f32) -> f32 {
    ((d - min)/(max-min)).max(0.).min(1.)
}

fn update_birds(    
    time: Res<Time>,
    players: Query<&Transform, (With<Player>, Without<Bird>)>,
    mut birds: Query<(Entity, &mut Transform, &mut Bird), With<Bird>>,
    bird_stats: Res<Birds>,
) {
    const MAX_STEERING_FORCE: f32 = 0.75;    
    const MAX_VELOCITY: f32 = 5.;        
    const MIN_DISTANCE: f32 = 0.5;     
    const MIN_VELOCITY: f32 = 0.;
    const MIN_AVOID_DISTANCE: f32 = 0.5;
    const MAX_AVOID_DISTANCE: f32 = 40.;
    const AVOID_FACTOR: f32 = 8.;
    const CENTER_FACTOR: f32 = 1.;
    const MIN_CENTER_DISTANCE: f32 = 50.;
    const MAX_CENTER_DISTANCE: f32 = 150.;

    const SEPERATION_FACTOR: f32 = 1.5;
    const ALIGNMENT_FACTOR: f32 = 0.70;
    const COHESION_FACTOR: f32 = 0.5;

    let delta = time.delta_seconds();
    let player = players.single();
    let mut kdtree: KdTree<f32, u32, 2, 32, u16> = KdTree::with_capacity(bird_stats.count);
    let mut positions: FxHashMap<u32, BirdData> = FxHashMap::with_capacity_and_hasher(bird_stats.count, Default::default());

    for (entity, transform, bird) in &mut birds {
        kdtree.add(&[transform.translation.x, transform.translation.y], entity.index());
        positions.insert(entity.index(), BirdData{
            translation: transform.translation.xy(), velocity: bird.velocity
        });
    }

    let mut count = 0;
    for (entity, mut transform, mut bird) in &mut birds {
        count+=1;
        let to_player = (player.translation - transform.translation).xy();
        let to_player_length = to_player.length();
        let avoid = -to_player.normalize_or_zero() * AVOID_FACTOR * (1. - strength(MIN_AVOID_DISTANCE, MAX_AVOID_DISTANCE, (to_player_length.clamp(MIN_AVOID_DISTANCE, MAX_AVOID_DISTANCE))));
        let center = -transform.translation.xy().normalize_or_zero() * CENTER_FACTOR * strength(MIN_CENTER_DISTANCE, MAX_CENTER_DISTANCE,transform.translation.xy().length());  
        let mut seperate = Vec2::new(0., 0.);
        let mut alignment = Vec2::new(0., 0.);
        let mut average_position = Vec2::new(0., 0.);
        let mut cohesion = Vec2::new(0., 0.);
        let mut nearest_count = 0;
        for n in kdtree.nearest_n_within::<SquaredEuclidean>(&[transform.translation.x, transform.translation.y], 5., 10, false) {
            if n.item == entity.index() {
                continue;
            }            
            if let Some(other) = positions.get(&n.item) {
                nearest_count += 1;
                let distance = n.distance.max(MIN_DISTANCE);
                let direction_from = (transform.translation.xy() - other.translation.xy()).normalize_or_zero();
                seperate += direction_from * (SEPERATION_FACTOR / distance);
                alignment += other.velocity.normalize_or_zero() * (ALIGNMENT_FACTOR / distance);
                average_position += other.translation.xy();
            }
        }
        if nearest_count > 0 {
            average_position = average_position / (nearest_count as f32);
            cohesion = (average_position - transform.translation.xy()).normalize_or_zero() * COHESION_FACTOR;
        }
        // TODO scale by timestep
        let mut steering_force = (
            avoid +
            seperate +
            alignment +
            cohesion +
            center
        ).clamp_length_max(MAX_STEERING_FORCE);        
        //eprintln!("Steering {}", steering_force);
        bird.velocity = (bird.velocity + steering_force).clamp_length(MIN_VELOCITY, MAX_VELOCITY);
        transform.translation = (transform.translation.xy() + bird.velocity * delta * 10.).extend(0.);
        
        if(transform.translation.x.is_nan() || transform.translation.y.is_nan()) {
          eprintln!("steering_force {} Bird v {} and p {}", steering_force, bird.velocity, transform.translation);
          panic!()
        }
    }    
}