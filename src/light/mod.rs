use bevy::{
    prelude::*,
    sprite::{AlphaMode2d, Material2dPlugin},
};

use enum_map::Enum;
use render::{LightMaterial, LightRenderData};
use segments::{
    cleanup_light_sources, simulate_light_sources, tick_light_sources, LightSegmentCache,
    PrevLightBeamPlayback,
};

use crate::{level::LevelSystems, shared::ResetLevel};

mod render;
pub mod segments;

/// The speed of the light beam in units per [`FixedUpdate`].
const LIGHT_SPEED: f32 = 8.0;

/// The width of the rectangle used to represent [`LightSegment`](segments::LightSegmentBundle)s.
const LIGHT_SEGMENT_THICKNESS: f32 = 3.0;

/// [`Plugin`] that manages everything light related.
pub struct LightManagementPlugin;

impl Plugin for LightManagementPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<LightMaterial>::default())
            .init_resource::<LightRenderData>()
            .init_resource::<LightSegmentCache>()
            .add_systems(
                FixedUpdate,
                (simulate_light_sources, tick_light_sources).in_set(LevelSystems::Simulation),
            )
            .add_systems(Update, cleanup_light_sources.run_if(on_event::<ResetLevel>));
    }
}

/// [`Enum`] for each of the light colors.
#[derive(Enum, Clone, Copy, Default, PartialEq, Debug, Eq, Hash)]
pub enum LightColor {
    #[default]
    Green,
    Red,
    White,
    Blue,
}

/// [`LightMaterial`] corresponding to each of the [`LightColor`]s.
impl From<LightColor> for LightMaterial {
    fn from(light_color: LightColor) -> Self {
        let color = light_color.light_beam_color();
        LightMaterial {
            color: color.into(),
            alpha_mode: AlphaMode2d::Blend,
            _wasm_padding: Vec2::ZERO,
        }
    }
}

impl From<&String> for LightColor {
    fn from(value: &String) -> Self {
        match value.as_str() {
            "Red" => LightColor::Red,
            "Green" => LightColor::Green,
            "White" => LightColor::White,
            "Blue" => LightColor::Blue,
            _ => panic!("String {} does not represent Light Color", value),
        }
    }
}

impl LightColor {
    /// The number of bounces off of terrain each [`LightColor`] can make.
    pub fn num_bounces(&self) -> usize {
        match self {
            LightColor::Red => 2,
            _ => 1,
        }
    }

    pub fn lighting_color(&self) -> Vec3 {
        match self {
            LightColor::Red => Vec3::new(1.0, 0.1, 0.1),
            LightColor::Green => Vec3::new(0.0, 1.0, 0.0),
            LightColor::White => Vec3::new(0.8, 0.8, 0.5),
            LightColor::Blue => Vec3::new(0.0, 0.0, 1.0),
        }
    }

    pub fn light_beam_color(&self) -> Color {
        match self {
            LightColor::Red => Color::srgb(5.0, 0.0, 3.0),
            LightColor::Green => Color::srgb(3.0, 5.0, 0.0),
            LightColor::White => Color::srgb(2.0, 2.0, 2.0),
            LightColor::Blue => Color::srgb(1.0, 2.0, 4.0),
        }
    }

    pub fn button_color(&self) -> Color {
        match self {
            LightColor::Red => Color::srgb(1.0, 0.5608, 0.8314),
            LightColor::Green => Color::srgb(0.6157, 0.9922, 0.5804),
            LightColor::White => Color::srgb(0.9, 0.9, 0.9),
            LightColor::Blue => Color::srgb(0.5608, 0.8824, 1.0),
        }
    }
}

/// A [`Component`] marking the start of a light ray. These are spawned in
/// [`shoot_light`](crate::player::light::shoot_light), and simulated in
/// [`simulate_light_sources`]
#[derive(Component)]
#[require(Transform, Visibility, Sprite, PrevLightBeamPlayback)]
pub struct LightBeamSource {
    pub start_pos: Vec2,
    pub start_dir: Vec2,
    pub time_traveled: f32,
    pub color: LightColor,
}
