use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::shared::GroupLabel;

/// Component for things that hurt
#[derive(Default, Component)]
pub struct HurtMarker;

/// Component for spikes
#[derive(Default, Component)]
pub struct Spike {
    // name: String,
    num_deaths: u32,
}

/// method to increase num_deaths of spike
impl Spike {
    pub fn add_death(&mut self) {
        self.num_deaths += 1;
    }
}

/// IntGrid implementation of Spike
impl From<IntGridCell> for Spike {
    fn from(cell_instance: IntGridCell) -> Self {
        match cell_instance.value {
            2 => Spike {
                // name: "baseSpike".to_string(),
                num_deaths: 0,
            },
            _ => unreachable!(),
        }
    }
}

/// Bundle for spikes
#[derive(Default, Bundle, LdtkIntCell)]
pub struct SpikeBundle {
    #[from_int_grid_cell]
    fixed_entity_bundle: FixedEntityBundle,
    hurt_marker: HurtMarker,
    spike: Spike,
}

/// [`Bundle`] used to group together components commonly used together when initializing physics
/// for fixed [`LdtkEntity`]s.
#[derive(Default, Bundle)]
pub struct FixedEntityBundle {
    collider: Collider,
    rigid_body: RigidBody,
    collision_groups: CollisionGroups,
}

/// IntGrid implementation of FixedEntityBundle
impl From<IntGridCell> for FixedEntityBundle {
    fn from(cell_instance: IntGridCell) -> Self {
        match cell_instance.value {
            2 => FixedEntityBundle {
                collider: Collider::triangle(
                    Vec2::new(-4., -4.),
                    Vec2::new(4., -4.),
                    Vec2::new(0., 4.),
                ),
                rigid_body: RigidBody::Fixed,
                collision_groups: CollisionGroups::new(
                    GroupLabel::TERRAIN,
                    GroupLabel::LIGHT_RAY
                        | GroupLabel::PLAYER_SENSOR
                        | GroupLabel::WHITE_RAY
                        | GroupLabel::STRAND,
                ),
            },
            _ => unreachable!(),
        }
    }
}
