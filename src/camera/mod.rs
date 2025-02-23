use std::time::Duration;

use bevy::{
    core_pipeline::tonemapping::Tonemapping,
    ecs::system::SystemId,
    prelude::*,
    render::{
        camera::{RenderTarget, ScalingMode},
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        view::RenderLayers,
    },
};
use bevy_rapier2d::plugin::PhysicsSet;

use crate::{
    level::{CurrentLevel, LevelSystems},
    player::PlayerMarker,
};

/// The [`Plugin`] responsible for handling anything Camera related.
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MoveCameraEvent>()
            .add_systems(Startup, setup_camera)
            .add_systems(
                FixedUpdate,
                move_camera
                    .after(PhysicsSet::Writeback)
                    .in_set(LevelSystems::Simulation),
            )
            // Has event reader, so place in update
            .add_systems(Update, (handle_move_camera, match_camera));
    }
}

/// Marker [`Component`] used to query for the main camera in the world.
///
/// Your query might look like this:
/// ```rust
/// Query<&Transform, With<MainCamera>>
/// ```
#[derive(Component, Default)]
pub struct MainCamera;

/// Marker [`Component`] used to query for the background camera. Note that for an entity to be
/// rendered on this Camera, it must be given the `RenderLayers::layer(1)` component.
#[derive(Component, Default)]
pub struct BackgroundCamera;

/// Marker [`Component`] used to query for the camera with pixel grid snapping support.
/// Note that for an entity to be rendered on this Camera, it must be given the
/// `RenderLayers::layer(2)` component.
#[derive(Component, Default)]
pub struct PixelGridSnapCamera;

/// Marker [`Component`] used to query for the camera with pixel grid snapping support.
/// Note that for an entity to be rendered on this Camera, it must be given the
/// `RenderLayers::layer(2)` component.
#[derive(Component, Default)]
pub struct PixelGridSnapCamera;

pub const CAMERA_WIDTH: f32 = 320.;
pub const CAMERA_HEIGHT: f32 = 180.;
pub const CAMERA_ANIMATION_SECS: f32 = 0.4;

/// [`Startup`] [`System`] that spawns the [`Camera2d`] in the world.
///
/// Notes:
/// - Spawns the camera with [`OrthographicProjection`] with fixed scaling at 320x180
fn setup_camera(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let projection = Projection::Orthographic(OrthographicProjection {
        scaling_mode: ScalingMode::Fixed {
            width: CAMERA_WIDTH,
            height: CAMERA_HEIGHT,
        },
        ..OrthographicProjection::default_2d()
    });

    // Set up for Low Resolution Canvas & Camera
    let lowres_canvas_size = Extent3d {
        width: CAMERA_WIDTH as u32,
        height: CAMERA_HEIGHT as u32,
        ..default()
    };

    let mut lowres_canvas = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            // Resolution for low res canvas should be smaller than camera resolutions
            size: lowres_canvas_size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    // fill image.data with zeroes to clear frame
    lowres_canvas.resize(lowres_canvas_size);
    let image_handle = images.add(lowres_canvas);

    commands
        .spawn((
            Camera2d,
            MainCamera,
            Camera {
                hdr: true,
                order: 1,
                clear_color: ClearColorConfig::None,
                ..default()
            },
            Tonemapping::TonyMcMapface,
            //Bloom::default(),
            projection.clone(),
            Transform::default(),
        ))
        .with_child((
            Sprite::from_image(image_handle.clone()),
            RenderLayers::layer(0),
        ));

    commands.spawn((
        Camera2d,
        PixelGridSnapCamera,
        Tonemapping::TonyMcMapface,
        Bloom::default(),
        Camera {
            hdr: true,
            target: RenderTarget::Image(image_handle.clone()),
            ..default()
        },
        projection.clone(),
        RenderLayers::layer(2),
        Transform::default(),
    ));

    // Setup for background camera
    commands.spawn((
        Camera2d,
        BackgroundCamera,
        Camera {
            order: -1,
            hdr: true, // If Cameras mix HDR and non-HDR, then weird ass stuff happens. Seems like
            // https://github.com/bevyengine/bevy/pull/13419 was only a partial fix
            ..default()
        },
        projection,
        RenderLayers::layer(1),
        Transform::default(),
    ));
}

fn match_camera(
    mut q_pixel: Query<&mut Transform, With<PixelGridSnapCamera>>,
    q_camera: Query<&Transform, (With<MainCamera>, Without<PixelGridSnapCamera>)>,
) {
    let Ok(camera) = q_camera.get_single() else {
        return;
    };
    let Ok(mut pixel) = q_pixel.get_single_mut() else {
        return;
    };
    *pixel = *camera;
}

fn match_camera(
    mut q_pixel: Query<&mut Transform, With<PixelGridSnapCamera>>,
    q_camera: Query<&Transform, (With<MainCamera>, Without<PixelGridSnapCamera>)>,
) {
    let Ok(camera) = q_camera.get_single() else {
        return;
    };
    let Ok(mut pixel) = q_pixel.get_single_mut() else {
        return;
    };
    *pixel = *camera;
}

#[derive(Event, Debug)]
pub enum MoveCameraEvent {
    Animated {
        to: Vec2,
        duration: Duration,
        // start and end use seconds
        curve: EasingCurve<f32>,
        callback: Option<SystemId>,
    },
    Instant {
        to: Vec2,
    },
}

pub struct Animation {
    progress: Timer,
    start: Vec3,
    end: Vec3,
    // start and end use seconds
    curve: EasingCurve<f32>,
    callback: Option<SystemId>,
}

pub fn handle_move_camera(
    mut commands: Commands,
    mut q_camera: Query<&mut Transform, With<MainCamera>>,
    mut ev_move_camera: EventReader<MoveCameraEvent>,
    mut animation: Local<Option<Animation>>,
    time: Res<Time>,
) {
    let Ok(mut camera_transform) = q_camera.get_single_mut() else {
        return;
    };

    for event in ev_move_camera.read() {
        match event {
            MoveCameraEvent::Animated {
                to,
                duration,
                curve,
                callback,
            } => {
                let anim = Animation {
                    progress: Timer::new(*duration, TimerMode::Once),
                    start: camera_transform.translation,
                    end: to.extend(camera_transform.translation.z),
                    curve: curve.clone(),
                    callback: *callback,
                };
                *animation = Some(anim);
            }
            MoveCameraEvent::Instant { to } => {
                camera_transform.translation = to.extend(camera_transform.translation.z);
            }
        }
    }

    // This is a reborrow, something that treats Bevy's "smart pointers" as actual Rust references,
    // which allows you to do the things you are supposed to (like pattern match on them).
    let Some(anim) = &mut *animation else {
        return;
    };

    anim.progress.tick(time.delta());

    let percent = anim.progress.elapsed_secs() / anim.progress.duration().as_secs_f32();
    camera_transform.translation = anim
        .start
        .lerp(anim.end, anim.curve.sample_clamped(percent));

    if anim.progress.just_finished() {
        if anim.callback.is_some() {
            commands.run_system(anim.callback.unwrap());
        }
        *animation = None;
    }
}

/// [`System`] that moves camera to player's position and constrains it to the [`CurrentLevel`]'s `world_box`.
pub fn move_camera(
    current_level: Res<CurrentLevel>,
    q_player: Query<&Transform, With<PlayerMarker>>,
    mut ev_move_camera: EventWriter<MoveCameraEvent>,
) {
    let Ok(player_transform) = q_player.get_single() else {
        return;
    };
    let (x_min, x_max) = (
        current_level.world_box.min.x + CAMERA_WIDTH * 0.5,
        current_level.world_box.max.x - CAMERA_WIDTH * 0.5,
    );
    let (y_min, y_max) = (
        current_level.world_box.min.y + CAMERA_HEIGHT * 0.5,
        current_level.world_box.max.y - CAMERA_HEIGHT * 0.5,
    );

    let new_pos = Vec2::new(
        player_transform.translation.x.max(x_min).min(x_max),
        player_transform.translation.y.max(y_min).min(y_max),
    );

    ev_move_camera.send(MoveCameraEvent::Instant { to: new_pos });
}
