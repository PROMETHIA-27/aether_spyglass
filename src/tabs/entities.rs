//! The entities tab module. Manages the inspector that selects entities, displays information
//! about them, and allows editing their components.

pub mod editors;

use bevy::prelude::*;
use bevy::utils::{HashMap, HashSet};
use bevy_egui::egui::{self, Ui};
use bevy_egui::EguiContexts;

use crate::{Spyglass, SpyglassWindow, Tab};

use self::editors::{
    array_editor, bool_editor, composite_editor, enum_editor, list_editor, map_editor, num_editor,
    string_editor, value_editor, EditorStates, VariantProxy,
};

/// The plugin that adds the entity tab to the inspector. Adds necessary resources, and
/// a few necessary systems, as well as adding the tab to the end of the [`Spyglass`] tab list.
pub struct EntitiesTabPlugin;

impl Plugin for EntitiesTabPlugin {
    fn build(&self, app: &mut App) {
        let mut spyglass = app.world.resource_mut::<Spyglass>();
        spyglass.tabs.push(Box::new(EntitiesTab));

        app.init_resource::<EntityTracker>()
            .init_resource::<EntitySearch>()
            .init_resource::<ReprEditors>()
            .init_resource::<EditorStates>()
            .init_resource::<Popups>()
            .add_systems(
                Update,
                (
                    (
                        display_popups,
                        collect_entity_state,
                        track_entities,
                        untrack_entities,
                    )
                        .chain()
                        .before(SpyglassWindow),
                    apply_entity_state.after(SpyglassWindow),
                ),
            );
    }
}

struct EntitiesTab;

impl Tab for EntitiesTab {
    fn name(&self) -> &str {
        "Entities"
    }

    fn draw(&mut self, ui: &mut Ui, world: &mut World) {
        let tracker = world.remove_resource::<EntityTracker>().unwrap();
        let mut search = world.remove_resource::<EntitySearch>().unwrap();
        let mut states = world.remove_resource::<EditorStates>().unwrap();

        if world.contains_resource::<SelectedEntity>() {
            draw_selection(ui, world, &mut states);
        } else {
            draw_no_selection(ui, world, &tracker, &mut search);
        }

        world.insert_resource(tracker);
        world.insert_resource(search);
        world.insert_resource(states);
    }
}

fn draw_selection(ui: &mut Ui, world: &mut World, states: &mut EditorStates) {
    if ui.button("back").clicked() {
        world.remove_resource::<SelectedEntity>();
        return;
    }

    let editors = world.remove_resource::<ReprEditors>().unwrap();
    let mut selected = world.remove_resource::<SelectedEntity>().unwrap();

    ui.group(|ui| {
        ui.vertical_centered(|ui| {
            ui.heading(&selected.name);
        });

        for comp in selected.state.components.iter() {
            if let Some(repr) = selected.state.reprs.get_mut(comp) {
                let editor = editors.get(repr.type_name());
                editor(ui, repr.as_mut(), world, &editors, states);
            } else {
                ui.label(comp).on_hover_ui(|ui| {
                    ui.label(
                        "No editable representation could be created for this component. \
                    Try implementing reflect for it, make sure to register its type with the app, \
                    and consider a TODO: custom representation.",
                    );
                });
            }
        }
    });

    world.insert_resource(editors);
    world.insert_resource(selected);
}

fn draw_no_selection(
    ui: &mut Ui,
    world: &mut World,
    tracker: &EntityTracker,
    search: &mut EntitySearch,
) {
    ui.vertical_centered(|ui| {
        egui::TextEdit::singleline(&mut search.0)
            .clip_text(false)
            .min_size(egui::vec2(ui.available_width() * 0.9, 0.0))
            .hint_text("Search for an entity")
            .show(ui);
    });

    for entity in tracker.tracked.iter().copied() {
        let name = world
            .get::<Name>(entity)
            .map(|name| name.to_string())
            .unwrap_or_else(|| format!("{entity:?}"));

        if !name.starts_with(&search.0) {
            continue;
        }

        if ui.button(&name).clicked() {
            let state = EntityComponents::from_entity(world, entity);
            world.insert_resource(SelectedEntity {
                id: entity,
                name,
                state,
            });
        }
    }
}

#[derive(Default, Resource)]
struct EntityTracker {
    tracked: HashSet<Entity>,
}

#[derive(Component)]
struct TrackedInSpyglass;

fn track_entities(
    mut c: Commands,
    q: Query<Entity, Without<TrackedInSpyglass>>,
    mut state: ResMut<EntityTracker>,
) {
    for entity in &q {
        c.entity(entity).insert(TrackedInSpyglass);
        state.tracked.insert(entity);
    }
}

fn untrack_entities(mut q: RemovedComponents<TrackedInSpyglass>, mut state: ResMut<EntityTracker>) {
    for entity in &mut q.read() {
        state.tracked.remove(&entity);
    }
}

struct EntityComponents {
    components: Vec<String>,
    reprs: HashMap<String, Box<dyn Reflect>>,
}

impl EntityComponents {
    fn from_entity(world: &World, entity: Entity) -> Self {
        let loc = world.entities().get(entity).unwrap();
        let archetype = world.archetypes().get(loc.archetype_id).unwrap();
        let mut components = vec![];
        let mut reprs = HashMap::default();
        for comp in archetype.components() {
            let name = if let Some(name) = world.components().get_name(comp) {
                if let Some(refl) = get_reflect_impl(world, name) {
                    if let Some(repr) = refl.reflect(world.entity(entity)) {
                        reprs.insert(name.to_string(), repr.clone_value());
                    }
                }
                name.to_string()
            } else if let Some(id) = world.components().get_info(comp).map(|info| info.type_id()) {
                format!("TypeId({id:?}")
            } else {
                format!("ComponentId({comp:?})")
            };

            components.push(name);
        }
        components.sort_unstable();
        Self { components, reprs }
    }
}

fn get_reflect_impl(world: &World, name: &str) -> Option<ReflectComponent> {
    let registry = world.get_resource::<AppTypeRegistry>()?.read();
    let registration = registry.get_with_short_type_path(name)?;
    registration.data::<ReflectComponent>().cloned()
}

#[derive(Resource)]
struct SelectedEntity {
    id: Entity,
    name: String,
    state: EntityComponents,
}

#[derive(Default, Resource)]
struct EntitySearch(String);

/// An editor of a given type. Arguments:
/// - `ui: &mut Ui`
/// - `repr: &mut dyn Reflect`
/// - `world: &mut World`
/// - `editors: &ReprEditors`
/// - `states: &mut EditorStates`
///
/// These can be created and added to [`ReprEditors`] to create custom editors for various types.
/// For example, primitive types are edited via specific [`ReprEditor`]s.
pub type ReprEditor =
    dyn Fn(&mut Ui, &mut dyn Reflect, &mut World, &ReprEditors, &mut EditorStates) + Send + Sync;

/// The resource that contains [`ReprEditor`]s, mapping from the
/// repr [`type_name`](std::any::type_name)s to their editor.
#[derive(Resource)]
pub struct ReprEditors {
    /// A map from [`type_name`](std::any::type_name)s to [`ReprEditor`].
    pub editors: HashMap<String, Box<ReprEditor>>,
}

impl Default for ReprEditors {
    fn default() -> Self {
        Self {
            editors: <_>::from([
                ("bool".to_string(), Box::new(bool_editor) as Box<ReprEditor>),
                ("i8".to_string(), Box::new(num_editor::<i8>)),
                ("i16".to_string(), Box::new(num_editor::<i16>)),
                ("i32".to_string(), Box::new(num_editor::<i32>)),
                ("i64".to_string(), Box::new(num_editor::<i64>)),
                ("isize".to_string(), Box::new(num_editor::<isize>)),
                ("u8".to_string(), Box::new(num_editor::<u8>)),
                ("u16".to_string(), Box::new(num_editor::<u16>)),
                ("u32".to_string(), Box::new(num_editor::<u32>)),
                ("u64".to_string(), Box::new(num_editor::<u64>)),
                ("usize".to_string(), Box::new(num_editor::<usize>)),
                ("f32".to_string(), Box::new(num_editor::<f32>)),
                ("f64".to_string(), Box::new(num_editor::<f64>)),
                ("alloc::string::String".to_string(), Box::new(string_editor)),
                (
                    std::any::type_name::<VariantProxy>().to_string(),
                    Box::new(VariantProxy::editor),
                ),
            ]),
        }
    }
}

impl ReprEditors {
    const REFLECT_EDITOR: &ReprEditor = &|ui, repr, world, editors, states| match repr.reflect_mut()
    {
        bevy::reflect::ReflectMut::Struct(repr) => {
            composite_editor(ui, repr, world, editors, states, false)
        }
        bevy::reflect::ReflectMut::TupleStruct(repr) => {
            composite_editor(ui, repr, world, editors, states, false)
        }
        bevy::reflect::ReflectMut::Tuple(repr) => {
            composite_editor(ui, repr, world, editors, states, false)
        }
        bevy::reflect::ReflectMut::List(repr) => list_editor(ui, repr, world, editors, states),
        bevy::reflect::ReflectMut::Array(repr) => array_editor(ui, repr, world, editors, states),
        bevy::reflect::ReflectMut::Map(repr) => map_editor(ui, repr, world, editors, states),
        bevy::reflect::ReflectMut::Enum(repr) => enum_editor(ui, repr, world, editors, states),
        bevy::reflect::ReflectMut::Value(repr) => value_editor(ui, repr),
    };

    /// Get an editor for a type based on its name. Returns either a custom [`ReprEditor`] or a
    /// default reflect-powered one if none exists.
    pub fn get(&self, name: &str) -> &ReprEditor {
        self.editors
            .get(name)
            .map(Box::as_ref)
            .unwrap_or(Self::REFLECT_EDITOR)
    }
}

fn collect_entity_state(world: &mut World) {
    let Some(SelectedEntity { id, name, state: _ }) = world.remove_resource::<SelectedEntity>() else { return };

    world.insert_resource(SelectedEntity {
        id,
        name,
        state: EntityComponents::from_entity(world, id),
    });
}

fn apply_entity_state(world: &mut World) {
    let Some(SelectedEntity { id, name, state }) = world.remove_resource::<SelectedEntity>() else { return };

    for (name, repr) in state.reprs.iter() {
        let refl = get_reflect_impl(world, name).unwrap();

        refl.apply(&mut world.entity_mut(id), &**repr);
    }

    world.insert_resource(SelectedEntity { id, name, state });
}

/// The resource that stores a list of current [`Popup`]s.
#[derive(Default, Resource)]
pub struct Popups {
    popups: Vec<Popup>,
}

impl Popups {
    /// Display the contained popups to the given [`egui::Context`].
    pub fn display_popups(&mut self, ui: &mut egui::Context) {
        let mut i = 0;
        loop {
            if i >= self.popups.len() {
                break;
            }

            let popup = &self.popups[i];
            if popup.display(i, ui) {
                self.popups.swap_remove(i);
            } else {
                i += 1;
            }
        }
    }

    /// Push a new popup onto the list.
    pub fn add(&mut self, popup: Popup) {
        self.popups.push(popup);
    }
}

/// A message popup, to be used with [`Popups`]. Commonly used for error messages.
pub struct Popup {
    message: String,
}

impl Popup {
    /// Create a new message popup.
    pub fn new(msg: impl Into<String>) -> Self {
        Popup {
            message: msg.into(),
        }
    }

    /// Display a popup to the given [`egui::Context`] with a given [`egui::Id`] source.
    pub fn display(&self, id: usize, ctx: &mut egui::Context) -> bool {
        let win = egui::Window::new("")
            .id(egui::Id::new("popup_window").with(id))
            .title_bar(false)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.label(&self.message);
                    ui.vertical_centered(|ui| ui.button("ok").clicked())
                })
            })
            .unwrap();
        win.response.clicked_elsewhere()
            || ctx.input(|inp| !inp.keys_down.is_empty())
            || win.inner.unwrap().inner.inner
    }
}

fn display_popups(mut egui: EguiContexts, mut popups: ResMut<Popups>) {
    popups.display_popups(egui.ctx_mut())
}
