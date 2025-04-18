use bevy::{math::vec2, prelude::*};
use bevy_rapier2d::prelude::*;

use crate::{
    animation::AnimationConfig, input::CursorWorldCoords, level::platform::cast_player_ray_shape,
    shared::GroupLabel,
};

use super::{light::PlayerLightInventory, movement::PlayerMovement, PlayerMarker};

pub const ANIMATION_FRAMES: usize = 29;

#[derive(Debug, Component, PartialEq, Eq, Clone, Copy, Default)]
pub enum PlayerAnimationType {
    #[default]
    Idle,
    Walk,
    Crouch,
    Jump,
    Fall,
    Land,
}

// HAIR, LEFT, RIGHT
const OFFSETS: [[Vec2; 3]; ANIMATION_FRAMES] = [
    [vec2(-2.0, 3.0), vec2(-3.0, -6.0), vec2(4.0, -6.0)], // idle 1
    [vec2(-2.0, 4.0), vec2(-3.0, -5.0), vec2(4.0, -5.0)],
    [vec2(-2.0, 4.0), vec2(-3.0, -5.0), vec2(4.0, -5.0)],
    [vec2(-3.0, 4.0), vec2(-4.0, -6.0), vec2(3.0, -6.0)], // walk 1
    [vec2(-2.0, 4.0), vec2(-3.0, -5.0), vec2(4.0, -5.0)],
    [vec2(-2.0, 4.0), vec2(-3.0, -5.0), vec2(4.0, -5.0)],
    [vec2(-3.0, 4.0), vec2(-2.0, -5.0), vec2(3.0, -5.0)],
    [vec2(-3.0, 4.0), vec2(-3.0, -5.0), vec2(4.0, -5.0)],
    [vec2(-2.0, 4.0), vec2(-4.0, -5.0), vec2(3.0, -5.0)],
    [vec2(-2.0, 4.0), vec2(-3.0, -4.0), vec2(3.0, -5.0)],
    [vec2(-3.0, 4.0), vec2(-3.0, -5.0), vec2(3.0, -5.0)],
    [vec2(-2.0, 3.0), vec2(-3.0, -6.0), vec2(4.0, -6.0)], // crouch 1
    [vec2(-2.0, 2.0), vec2(-3.0, -7.0), vec2(4.0, -6.0)],
    [vec2(-2.0, 1.0), vec2(-3.0, -8.0), vec2(4.0, -7.0)],
    [vec2(-2.0, 0.0), vec2(-3.0, -8.0), vec2(4.0, -8.0)],
    [vec2(-2.0, 2.0), vec2(-3.0, -7.0), vec2(4.0, -6.0)], // jump 1
    [vec2(-2.0, 1.0), vec2(-3.0, -8.0), vec2(4.0, -7.0)],
    [vec2(-2.0, 3.0), vec2(-3.0, -6.0), vec2(3.0, -6.0)],
    [vec2(-2.0, 4.0), vec2(-3.0, -5.0), vec2(3.0, -5.0)],
    [vec2(-1.0, 4.0), vec2(-2.0, -5.0), vec2(3.0, -5.0)],
    [vec2(-2.0, 4.0), vec2(-3.0, -5.0), vec2(3.0, -5.0)],
    [vec2(-2.0, 4.0), vec2(-3.0, -4.0), vec2(3.0, -4.0)], // fall 1
    [vec2(-2.0, 4.0), vec2(-4.0, -4.0), vec2(4.0, -4.0)],
    [vec2(-2.0, 4.0), vec2(-4.0, -3.0), vec2(4.0, -3.0)],
    [vec2(-2.0, 4.0), vec2(-5.0, -2.0), vec2(5.0, -2.0)],
    [vec2(-2.0, 4.0), vec2(-4.0, -3.0), vec2(5.0, -3.0)], // land 1
    [vec2(-2.0, 3.0), vec2(-3.0, -4.0), vec2(4.0, -4.0)],
    [vec2(-2.0, 1.0), vec2(-4.0, -5.0), vec2(5.0, -5.0)],
    [vec2(-2.0, 3.0), vec2(-4.0, -4.0), vec2(4.0, -4.0)],
];

impl PlayerAnimationType {
    fn get_offset(&self, index: usize, variant: usize) -> Vec2 {
        OFFSETS[index][variant]
    }
    pub fn hair_offset(&self, index: usize) -> Vec2 {
        self.get_offset(index, 0)
    }
    pub fn left_cloth_offset(&self, index: usize) -> Vec2 {
        self.get_offset(index, 1)
    }
    pub fn right_cloth_offset(&self, index: usize) -> Vec2 {
        self.get_offset(index, 2)
    }
}

impl From<PlayerAnimationType> for AnimationConfig {
    fn from(anim_type: PlayerAnimationType) -> Self {
        match anim_type {
            PlayerAnimationType::Walk => AnimationConfig::new(3, 10, 12, true),
            PlayerAnimationType::Idle => AnimationConfig::new(0, 2, 6, true),
            PlayerAnimationType::Crouch => AnimationConfig::new(11, 14, 48, false),
            PlayerAnimationType::Jump => AnimationConfig::new(15, 20, 24, false),
            PlayerAnimationType::Fall => AnimationConfig::new(21, 24, 24, false),
            PlayerAnimationType::Land => AnimationConfig::new(25, 28, 18, false),
        }
    }
}

pub fn flip_player_direction(
    mut q_player: Query<
        (
            &mut Sprite,
            &KinematicCharacterControllerOutput,
            &GlobalTransform,
            &PlayerLightInventory,
        ),
        With<PlayerMarker>,
    >,
    buttons: Res<ButtonInput<MouseButton>>,
    q_cursor: Query<&CursorWorldCoords>,
) {
    let Ok((mut player_sprite, player_controller_output, player_transform, player_light_inventory)) =
        q_player.get_single_mut()
    else {
        return;
    };
    let Ok(cursor_coords) = q_cursor.get_single() else {
        return;
    };

    if buttons.pressed(MouseButton::Left) && player_light_inventory.can_shoot() {
        let to_cursor = cursor_coords.pos - player_transform.translation().xy();
        player_sprite.flip_x = to_cursor.x < 0.0;
        return;
    }

    const PLAYER_FACING_EPSILON: f32 = 0.01;
    if player_controller_output.desired_translation.x < -PLAYER_FACING_EPSILON {
        player_sprite.flip_x = true;
    } else if player_controller_output.desired_translation.x > PLAYER_FACING_EPSILON {
        player_sprite.flip_x = false;
    }
}

pub fn set_animation(
    mut q_player: Query<
        (
            &PlayerMovement,
            &mut AnimationConfig,
            &mut PlayerAnimationType,
            &Transform,
            &KinematicCharacterControllerOutput,
        ),
        With<PlayerMarker>,
    >,
    mut was_grounded: Local<bool>,
    rapier_context: ReadDefaultRapierContext<'_, '_>,
) {
    let Ok((movement, mut config, mut animation, transform, output)) = q_player.get_single_mut()
    else {
        return;
    };

    let entity_below_player = cast_player_ray_shape(
        &rapier_context,
        transform,
        0.0,
        -11.0,
        16.0,
        2.0,
        Vec2::new(0.0, -1.0),
        GroupLabel::PLATFORM,
    );

    let new_anim = if !output.grounded && output.effective_translation.y > 0.0 {
        PlayerAnimationType::Jump
    } else if !output.grounded && entity_below_player.is_none() {
        PlayerAnimationType::Fall
    } else if output.grounded && !*was_grounded {
        PlayerAnimationType::Land
    } else if output.grounded && output.effective_translation.x.abs() > 0.05 {
        PlayerAnimationType::Walk
    } else if output.grounded && movement.crouching {
        PlayerAnimationType::Crouch
    } else {
        PlayerAnimationType::Idle
    };

    if new_anim != *animation {
        // don't switch the animation out of falling if it isn't finished
        // there is probably a better way to do this :'(
        let should_cancel_animation = *animation != PlayerAnimationType::Land || config.finished;

        if should_cancel_animation {
            *animation = new_anim;
            *config = AnimationConfig::from(new_anim);
        }
    }
    *was_grounded = output.grounded || entity_below_player.is_some();
}
