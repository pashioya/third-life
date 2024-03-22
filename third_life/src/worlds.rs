pub mod config;
pub(crate) mod food;
pub(crate) mod population;
pub(crate) mod ui;
pub(crate) mod env_and_infra;
pub(crate) mod wealth;

use bevy::prelude::*;

use crate::{
    animation::ColonyAnimationBundle,
    config::SelectedConfigPath,
    SimulationState,
};

use self::{
    config::{WorldConfig, WorldsConfig, WorldsConfigPlugin},
    env_and_infra::{components::ColonyInfraAndEnvBundle, InfrastructurePlugin},
    food::FoodPlugin,
    population::{
        components::Population,
        PopulationPlugin,
    },
    ui::WorldsUiPlugin,
    wealth::{
        components::ColonyWealthBundle,
        WealthPlugin,
    },
};

pub struct WorldsPlugin;

impl Plugin for WorldsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(SimulationState::Running), init_colonies)
            .add_plugins((
                WorldsConfigPlugin,
                PopulationPlugin,
                FoodPlugin,
                WorldsUiPlugin,
                InfrastructurePlugin,
                WealthPlugin,
            ));
    }
}

pub(crate) fn init_colonies(
    mut commands: Commands,
    worlds_config: Res<WorldsConfig>,
    asset_server: Res<AssetServer>,
    config_path: Res<SelectedConfigPath>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let font = asset_server.load("fonts/VictorMonoNerdFontMono-Medium.ttf");
    for world in worlds_config.worlds() {
        let texture = asset_server.load(format!(
            "{}/sprite_sheets/{}",
            config_path.0,
            world.sprite().sprite_sheet()
        ));
        commands
            .spawn(WorldColonyBundle::new(
                texture,
                &mut texture_atlas_layouts,
                world.clone(),
            ))
            .with_children(|parent| {
                parent.spawn(Text2dBundle {
                    text: Text::from_section(
                        world.name(),
                        TextStyle {
                            font: font.clone(),
                            font_size: 24.,
                            color: Color::WHITE,
                        },
                    )
                    .with_justify(JustifyText::Center),
                    transform: Transform::from_xyz(0., -1. * world.sprite().shape().0 as f32, 0.),
                    ..default()
                });
            });
    }
}

#[derive(Component, PartialEq)]
pub struct WorldColony {
    size: f32,
    used: f32,
}

impl WorldColony {
    fn new(starting_size: f32) -> Self {
        WorldColony {
            size: starting_size,
            used: 0.0,
        }
    }

    fn space_left(&self) -> f32 {
        self.size - self.used
    }
}

#[derive(Component)]
pub struct WorldEntity {
    pub name: String,
}

impl WorldEntity {
    fn new(name: String) -> Self {
        WorldEntity { name }
    }
}

#[derive(Component)]
pub struct ResourceAmount(f64);

#[derive(Bundle)]
pub struct WorldColonyBundle {
    colony: WorldColony,
    entity: WorldEntity,
    population: Population,
    animation: ColonyAnimationBundle,
    wealth: ColonyWealthBundle,
    infra_and_env: ColonyInfraAndEnvBundle,
    config: WorldConfig,
}

impl WorldColonyBundle {
    pub fn new(
        sprite_sheet: Handle<Image>,
        texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
        world: WorldConfig,
    ) -> Self {
        Self {
            colony: WorldColony::new(world.size()),
            entity: WorldEntity::new(world.name()),
            population: Population::default(),
            animation: ColonyAnimationBundle::new(
                world.world_position(),
                sprite_sheet,
                texture_atlas_layouts,
                world.sprite(),
            ),
            wealth: ColonyWealthBundle::new(world.government()),
            infra_and_env: ColonyInfraAndEnvBundle::default(),
            config: world,
        }
    }
}
