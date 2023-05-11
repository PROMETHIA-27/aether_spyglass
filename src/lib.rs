pub mod tabs;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::egui::{ScrollArea, Ui};
use bevy_egui::{egui, EguiContext, EguiPlugin};
use tabs::entities::EntitiesTabPlugin;

pub struct SpyglassPlugin;

impl Plugin for SpyglassPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(EguiPlugin)
            .init_resource::<Spyglass>()
            .add_system(spyglass_window.in_set(SpyglassWindow))
            .add_plugin(EntitiesTabPlugin);
    }
}

pub trait Tab: Send + Sync {
    fn name(&self) -> &str;

    fn draw(&mut self, ui: &mut Ui, world: &mut World);
}

#[derive(Default, Resource)]
pub struct Spyglass {
    pub tabs: Vec<Box<dyn Tab>>,
    selected: Option<usize>,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, SystemSet)]
pub struct SpyglassWindow;

fn spyglass_window(world: &mut World) {
    let Ok(primary_window) = world
        .query_filtered::<Entity, With<PrimaryWindow>>()
        .get_single(world)
        else { return };

    let Some(mut ctx) = world.entity_mut(primary_window).take::<EguiContext>() else { return };

    let mut state = world.remove_resource::<Spyglass>().unwrap();

    egui::Window::new("Spyglass").show(ctx.get_mut(), |ui| {
        egui::menu::bar(ui, |ui| {
            let mut selected = state.selected;
            for (i, tab) in state.tabs.iter().enumerate() {
                if ui
                    .selectable_label(selected == Some(i), tab.name())
                    .clicked()
                {
                    selected = if selected == Some(i) { None } else { Some(i) };
                }
            }
            state.selected = selected;
        });

        ui.separator();

        match state.selected {
            Some(selected) => {
                let Some(tab) = state.tabs.get_mut(selected) else {
                    state.selected = None;
                    return;
                };

                ScrollArea::new([true, true]).show(ui, |ui| {
                    tab.draw(ui, world);
                });
            }
            None => {
                ui.heading("Please select a tab to inspect.");
            }
        }
    });

    world.insert_resource(state);
    world.entity_mut(primary_window).insert(ctx);
}
