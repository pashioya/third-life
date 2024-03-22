
use crate::{worlds::{ui::components::*, population::{events::{CitizenCreated, CitizenDied, DeathReason}, components::{Citizen, CitizenOf, Population}}}, SimulationState};
use std::collections::HashMap;
use bevy::prelude::*;
use bevy_egui::egui::{Color32, Ui};
use egui_plot::{Plot, BarChart, Legend, Bar, Line};
use crate::time::GameDate;

use super::usize_to_plotpoints;

pub struct PopulationUiPlugin;

impl Plugin for PopulationUiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (
                    add_citizens_to_population_histogram,
                    update_ages,
                    update_general_pop,
                    death_events_listener
            ).run_if(in_state(SimulationState::Running)));
    }
}

pub fn add_citizens_to_population_histogram(
    mut pop_histograms: Query<(&WorldUiEntity, &mut PopulationUiData)>,
    mut citizen_created: EventReader<CitizenCreated>,
) {
    let mut map = pop_histograms.iter_mut().map(|(e, p)| (e.0, p)).collect::<HashMap<_, _>>();
    for CitizenCreated { age, colony, mother_age } in citizen_created.read() {
        let Some(pop_data) = map.get_mut(colony) else {
            continue;
        };
        *pop_data.ages
            .entry(*age)
            .or_insert(0) += 1;
        if let Some(mother_age) = mother_age {
            *pop_data.births_per_age
                .entry(*mother_age)
                .or_insert(0) += 1;
            pop_data.total_births += 1;
        }
    }
}

pub fn update_ages(
    citizens: Query<(&Citizen, &CitizenOf)>,
    game_date: Res<GameDate>,
    mut populations: Query<(&WorldUiEntity, &mut PopulationUiData)>
) {
    let map = populations.iter_mut().map(|(e, p)| (e.0, p)).collect::<HashMap<_, _>>();
    let populations_map = citizens.into_iter().fold(
        HashMap::new(),
        |mut acc: HashMap<Entity, HashMap<usize, usize>>, (citizen, citizen_of)| {
            *acc.entry(citizen_of.colony)
                .or_insert(HashMap::new())
                .entry(game_date.date.years_since(citizen.birthday).unwrap() as usize)
                .or_insert(0) += 1;
            acc
        },
    );

    for (key, mut val) in map.into_iter() {
        match populations_map.get(&key) {
            Some(new_map) => val.ages = new_map.clone(),
            None => ()
        }
    }
}

pub fn update_general_pop(
    query: Query<(Entity, &Population)>,
    mut populations: Query<(&WorldUiEntity, &mut PopulationUiData)>
) {
    let mut map = populations.iter_mut().map(|(e, p)| (e.0, p)).collect::<HashMap<_, _>>();
    for (col, population) in query.iter() {
        match map.get_mut(&col) {
            Some(p) => {
                p.count = population.count;
                p.working_pop = population.working_pop;
                p.younglings = population.younglings;
                p.retirees = population.retirees;
                p.average_age = population.average_age;
                p.average_children_per_mother = population.average_children_per_mother;
                p.num_couples = population.num_couples;
                p.males = population.males;
                p.females = population.females;
                p.working_in_wheat = population.working_in_wheat;
                p.working_in_beef = population.working_in_beef;
                p.working_not_in_farming = population.working_not_in_farming;
            }
            None => ()
        }
    }
}

pub fn general_pop(
    ui: &mut Ui,
    pop: &PopulationUiData,
) {
    ui.group(|ui|{
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(format!("Average Age: {:?}", pop.average_age));
                ui.label(format!("Total pop: {:?}",pop.count));
                ui.label(format!("Employable Pop: {:?}",pop.working_pop));
            });
            ui.vertical(|ui| {
                ui.label(format!("Younglings: {:?}",pop.younglings));
                ui.label(format!("Retirees: {:?}",pop.retirees));
            });
            ui.vertical(|ui| {
                ui.label(format!("Wheat workers: {:?}",pop.working_in_wheat));
                ui.label(format!("Beef workers: {:?}", pop.working_in_beef));
                ui.label(format!("Working not in farming: {:?}", pop.working_not_in_farming));
            });
        });
    });
    ui.group(|ui|{
        ui.vertical(|ui| {
            ui.label(format!("Couples: {:?}",pop.num_couples));
            ui.label(format!("Average Children per Mother: {:.2}", pop.average_children_per_mother));
            ui.label(format!("Total Births: {:.2}", pop.total_births));
            ui.label(format!("Females: {:?}",pop.females));
            ui.label(format!("Males: {:?}",pop.males));
        });
    });
}

pub fn age_histogram(
    planet_name: &str,
    ui: &mut Ui,
    ages: &HashMap<usize, usize>
) {
    let bars = (0..100)
        .into_iter()
        .map(|index| {
            let height = ages.get(&index).map(|u| *u).unwrap_or(0);
            Bar::new(index as f64, height as f64).width(1.)
        })
    .collect::<Vec<_>>();
    let chart = BarChart::new(bars)
        .color(Color32::LIGHT_BLUE)
        .name(format!("Population chart of {planet_name}"));
    Plot::new(format!("Population {planet_name}"))
        .legend(Legend::default())
        .height(200.)
        .y_axis_width(3)
        .allow_zoom(false)
        .allow_drag(false)
        .allow_scroll(false)
        .show(ui, |plot_ui| plot_ui.bar_chart(chart))
        .response;
}

pub fn death_lines(
    planet_name: &str,
    ui: &mut Ui,
    deaths: &PopulationDeathLines
) {
    Plot::new(format!("Deaths on planet {planet_name}"))
        .allow_zoom(false)
        .allow_scroll(false)
        .allow_drag(false)
        .height(150.)
        .legend(Legend::default())
        .allow_zoom(false).allow_scroll(false).allow_drag(false)
        .show(ui, |plot_ui| {
            plot_ui.line(
                Line::new(usize_to_plotpoints(&deaths.old_age_deaths))
                    .color(Color32::from_rgb(0, 0, 255))
                    .name("old age")
            );
            plot_ui.line(
                Line::new(usize_to_plotpoints(&deaths.starvation_deaths))
                    .color(Color32::from_rgb(255, 0, 0))
                    .name("starvation")
            );
            plot_ui.line(
                Line::new(usize_to_plotpoints(&deaths.infant_deaths))
                    .color(Color32::from_rgb(0, 255, 0))
                    .name("infant death")
            );
        });
}

pub fn death_events_listener(
    time: Res<Time>,
    mut events: EventReader<CitizenDied>,
    mut uis: Query<(&WorldUiEntity, &mut PopulationDeathLines)>
) {
    let mapped_events = events.read().into_iter()
        .fold(HashMap::new(), |mut acc: HashMap<Entity, (usize, usize, usize)>, e| {
            let col_map = acc.entry(e.colony).or_insert((0, 0, 0));
            match e.reason {
                DeathReason::OldAge => col_map.0 += 1,
                DeathReason::Starvation => col_map.1 += 1,
                DeathReason::InfantDeath => col_map.2 += 1,
            };
            acc
        });

    for (WorldUiEntity(colony), mut lines) in uis.iter_mut() {
        lines.new_step(time.delta());
        let (old_age, starvation, infant) = mapped_events.get(&colony)
            .map(|(old_age, starvation, infant)|(*old_age, *starvation, *infant))
            .unwrap_or((0, 0, 0));
        *lines.old_age_deaths.last_mut().unwrap() += old_age;
        *lines.starvation_deaths.last_mut().unwrap() += starvation;
        *lines.infant_deaths.last_mut().unwrap() += infant;
    }
}
