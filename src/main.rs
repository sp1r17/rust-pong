use bevy::{
    // app::AppExit,
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
    window::PresentMode,
};

const WINDOW_WIDTH: f32 = 1280.0;
const WINDOW_HEIGHT: f32 = 720.0;

const PADDLE_SIZE: Vec3 = Vec3::new(20.0, 120.0, 0.0);
const PADDLE_SPEED: f32 = 400.0;
const PADDLE_PADDING: f32 = 20.0;

const BALL_STARTING_POSITION: Vec3 = Vec3::new(0.0, 0.0, 1.0);
const BALL_SIZE: Vec3 = Vec3::new(20.0, 20.0, 0.0);
const BALL_SPEED: f32 = 500.0;
const INITIAL_BALL_DIRECTION: Vec2 = Vec2::new(0.5, -0.5);

const WALL_THICKNESS: f32 = 10.0;
const LEFT_WALL: f32 = -640.0;
const RIGHT_WALL: f32 = 640.0;
const BOTTOM_WALL: f32 = -360.0;
const TOP_WALL: f32 = 360.0;

#[derive(Component)]
pub struct LeftPaddle;

#[derive(Component)]
pub struct RightPaddle;

#[derive(Component)]
pub struct Ball;

#[derive(Component, Deref, DerefMut, Debug)]
struct Velocity(Vec2);

#[derive(Component)]
struct Collider;

#[derive(Event, Default)]
struct CollisionEvent;

#[derive(Resource)]
struct Scoreboard {
    left_player_score: usize,
    right_player_score: usize,
}

#[derive(Bundle)]
struct WallBundle {
    sprite_bundle: SpriteBundle,
    collider: Collider,
}

#[derive(Component)]
enum WallLocation {
    // Left,
    // Right,
    Bottom,
    Top,
}

impl WallLocation {
    fn position(&self) -> Vec2 {
        match self {
            // WallLocation::Left => Vec2::new(LEFT_WALL, 0.0),
            // WallLocation::Right => Vec2::new(RIGHT_WALL, 0.0),
            WallLocation::Bottom => Vec2::new(0.0, BOTTOM_WALL),
            WallLocation::Top => Vec2::new(0.0, TOP_WALL),
        }
    }

    fn size(&self) -> Vec2 {
        let arena_height = TOP_WALL - BOTTOM_WALL;
        let arena_width = RIGHT_WALL - LEFT_WALL;
        assert!(arena_height > 0.0);
        assert!(arena_width > 0.0);

        match self {
            // WallLocation::Left | WallLocation::Right => {
            //     Vec2::new(WALL_THICKNESS, arena_height + WALL_THICKNESS)
            // }
            WallLocation::Bottom | WallLocation::Top => {
                Vec2::new(arena_width + WALL_THICKNESS, WALL_THICKNESS)
            }
        }
    }
}

impl WallBundle {
    fn new(location: WallLocation) -> WallBundle {
        WallBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: location.position().extend(0.0),
                    scale: location.size().extend(1.0),
                    ..default()
                },
                ..default()
            },
            collider: Collider,
        }
    }
}

fn main() {
    App::new()
        .insert_resource(Msaa::Off)
        .insert_resource(ClearColor(Color::BLACK))
        // Cap tick rate
        .insert_resource(FixedTime::new_from_secs(1.0 / 60.0))
        .insert_resource(Scoreboard {
            left_player_score: 0,
            right_player_score: 0,
        })
        .add_event::<CollisionEvent>()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Rust Pong".to_string(),
                    resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                    decorations: false,
                    // Bind to canvas included in `index.html`
                    canvas: Some("#bevy".to_owned()),
                    // Tells wasm not to override default event handling, like F5 and Ctrl+R
                    prevent_default_event_handling: false,
                    present_mode: PresentMode::AutoNoVsync,
                    ..default()
                }),
                ..default()
            }), // Make pixel perfect
                // .set(ImagePlugin::default_nearest()),
        )
        .add_systems(Startup, init)
        .add_systems(
            FixedUpdate,
            (
                check_for_collisions,
                apply_velocity.before(check_for_collisions),
                move_left_paddle
                    .before(check_for_collisions)
                    .after(apply_velocity),
                move_right_paddle
                    .before(check_for_collisions)
                    .after(apply_velocity),
            ),
        )
        .add_systems(
            Update,
            (
                bevy::window::close_on_esc,
                check_if_ball_on_screen,
                update_scoreboard,
            ),
        )
        .run();
}

fn init(mut commands: Commands, _asset_server: Res<AssetServer>) {
    // let window_res = get_window_res(window);

    // Camera
    commands.spawn(Camera2dBundle::default());

    // Left Paddle
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new((-WINDOW_WIDTH / 2.0) + (PADDLE_SIZE.x * 2.0), 0.0, 0.0),
                scale: PADDLE_SIZE,
                ..default()
            },
            sprite: Sprite {
                color: Color::rgb(1.0, 1.0, 1.0),
                ..default()
            },
            ..default()
        },
        LeftPaddle,
        Collider,
    ));

    // Right Paddle
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new((WINDOW_WIDTH / 2.0) - (PADDLE_SIZE.x * 2.0), 0.0, 0.0),
                scale: PADDLE_SIZE,
                ..default()
            },
            sprite: Sprite {
                color: Color::rgb(1.0, 1.0, 1.0),
                ..default()
            },
            ..default()
        },
        RightPaddle,
        Collider,
    ));

    // Ball
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: BALL_STARTING_POSITION,
                scale: BALL_SIZE,
                ..default()
            },
            sprite: Sprite {
                color: Color::rgb(1.0, 1.0, 1.0),
                ..default()
            },
            ..default()
        },
        Ball,
        Velocity(INITIAL_BALL_DIRECTION.normalize() * BALL_SPEED),
    ));

    // Scoreboard
    commands.spawn(
        TextBundle::from_sections([
            TextSection::new(
                "0 : 0",
                TextStyle {
                    font_size: 30.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            TextSection::from_style(TextStyle {
                font_size: 30.0,
                color: Color::WHITE,
                ..default()
            }),
        ])
        .with_text_alignment(TextAlignment::Center)
        .with_style(Style {
            position_type: PositionType::Absolute,
            left: Val::Px(WINDOW_WIDTH / 2.0 - 35.0),
            top: Val::Px(15.0),
            ..default()
        }),
    );

    // Walls
    // commands.spawn(WallBundle::new(WallLocation::Left));
    // commands.spawn(WallBundle::new(WallLocation::Right));
    commands.spawn(WallBundle::new(WallLocation::Bottom));
    commands.spawn(WallBundle::new(WallLocation::Top));
}

// fn check_inputs(mut app_exit: EventWriter<AppExit>, keyboard_event: Res<Input<KeyCode>>) {
//     if keyboard_event.pressed(KeyCode::Escape) {
//         app_exit.send(AppExit);
//     }
// }

fn move_left_paddle(
    keyboard_event: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<LeftPaddle>>,
    time_step: Res<FixedTime>,
) {
    let mut paddle_transform = query.single_mut();
    let mut direction = 0.0;

    if keyboard_event.pressed(KeyCode::W) {
        direction += 1.0;
    } else if keyboard_event.pressed(KeyCode::S) {
        direction -= 1.0;
    }

    let new_paddle_position =
        paddle_transform.translation.y + direction * PADDLE_SPEED * time_step.period.as_secs_f32();

    let top_bound = WINDOW_HEIGHT / 2.0 - PADDLE_SIZE.y / 2.0 - PADDLE_PADDING;
    let bottom_bound = -WINDOW_HEIGHT / 2.0 + PADDLE_SIZE.y / 2.0 + PADDLE_PADDING;

    paddle_transform.translation.y = new_paddle_position.clamp(bottom_bound, top_bound);
}

fn move_right_paddle(
    keyboard_event: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<RightPaddle>>,
    time_step: Res<FixedTime>,
) {
    let mut paddle_transform = query.single_mut();
    let mut direction = 0.0;

    if keyboard_event.pressed(KeyCode::Up) {
        direction += 1.0;
    } else if keyboard_event.pressed(KeyCode::Down) {
        direction -= 1.0;
    }

    let new_paddle_position =
        paddle_transform.translation.y + direction * PADDLE_SPEED * time_step.period.as_secs_f32();

    let top_bound = WINDOW_HEIGHT / 2.0 - PADDLE_SIZE.y / 2.0 - PADDLE_PADDING;
    let bottom_bound = -WINDOW_HEIGHT / 2.0 + PADDLE_SIZE.y / 2.0 + PADDLE_PADDING;

    paddle_transform.translation.y = new_paddle_position.clamp(bottom_bound, top_bound);
}

fn apply_velocity(mut query: Query<(&mut Transform, &mut Velocity)>, time_step: Res<FixedTime>) {
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.x * time_step.period.as_secs_f32();
        transform.translation.y += velocity.y * time_step.period.as_secs_f32();
    }
}

fn update_scoreboard(scoreboard: Res<Scoreboard>, mut query: Query<&mut Text>) {
    let mut text = query.single_mut();
    text.sections[0].value = String::from(
        scoreboard.left_player_score.to_string()
            + " : "
            + &scoreboard.right_player_score.to_string(),
    );
}

fn check_for_collisions(
    // mut commands: Commands,
    mut ball_query: Query<(&mut Velocity, &Transform), With<Ball>>,
    collider_query: Query<(Entity, &Transform), With<Collider>>,
    mut collision_events: EventWriter<CollisionEvent>,
) {
    let (mut ball_velocity, ball_transform) = ball_query.single_mut();
    let ball_size = ball_transform.scale.truncate();

    for (_, transform) in &collider_query {
        let collision = collide(
            ball_transform.translation,
            ball_size,
            transform.translation,
            transform.scale.truncate(),
        );
        if let Some(collision) = collision {
            collision_events.send_default();

            let mut reflect_x = false;
            let mut reflect_y = false;

            match collision {
                Collision::Left => reflect_x = ball_velocity.x > 0.0,
                Collision::Right => reflect_x = ball_velocity.x < 0.0,
                Collision::Top => reflect_y = ball_velocity.y < 0.0,
                Collision::Bottom => reflect_y = ball_velocity.y > 0.0,
                Collision::Inside => { /* do nothing */ }
            }

            if reflect_x {
                ball_velocity.x = -ball_velocity.x;
            }

            if reflect_y {
                ball_velocity.y = -ball_velocity.y;
            }
        }
    }
}

fn check_if_ball_on_screen(
    mut scoreboard: ResMut<Scoreboard>,
    mut ball_query: Query<&mut Transform, With<Ball>>,
) {
    let mut ball_transform = ball_query.single_mut();
    let ball_size = ball_transform.scale.truncate();

    if ball_transform.translation.x + ball_size.x / 2.0 > WINDOW_WIDTH / 2.0 {
        ball_transform.translation = BALL_STARTING_POSITION;
        scoreboard.left_player_score += 1;
    } else if ball_transform.translation.x - ball_size.x / 2.0 < -WINDOW_WIDTH / 2.0 {
        ball_transform.translation = BALL_STARTING_POSITION;
        scoreboard.right_player_score += 1;
    }
}
