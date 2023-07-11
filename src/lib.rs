#![forbid(missing_docs, rustdoc::broken_intra_doc_links)]
#![doc = include_str!("../README.md")]

pub mod tabs;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::egui::{ScrollArea, Ui};
use bevy_egui::{egui, EguiContext, EguiPlugin};
use tabs::entities::EntitiesTabPlugin;

/// The main plugin used to add the spyglass inspector to an app.
/// Automatically adds the [`EguiPlugin`], creates the [`Spyglass`] resource,
/// the [`SpyglassWindow`] system set, and inserts the [`EntitiesTabPlugin`].
pub struct SpyglassPlugin;

impl Plugin for SpyglassPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(EguiPlugin)
            .init_resource::<Spyglass>()
            .add_systems(Update, spyglass_window.in_set(SpyglassWindow))
            .add_plugins(EntitiesTabPlugin);
    }
}

/// The trait to implement to create a new tab in the spyglass inspector.
pub trait Tab: Send + Sync {
    /// Returns the name of the tab, which will be displayed in the inspector.
    fn name(&self) -> &str;

    /// Draw the tab.
    fn draw(&mut self, ui: &mut Ui, world: &mut World);
}

/// The resource for managing the spyglass inspector.
#[derive(Default, Resource)]
pub struct Spyglass {
    /// Contains the ordered list of tabs to display.
    /// May be modified at any time to alter what is displayed.
    pub tabs: Vec<Box<dyn Tab>>,
    /// Contains the index of what tab is selected, if any.
    /// May be altered at any time, for example as an implementation of hotkeys.
    pub selected: Option<usize>,
}

/// The system set that draws the spyglass window. A good anchor point if there are
/// systems to be run as part of a tab.
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
