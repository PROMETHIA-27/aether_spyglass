use aether_spyglass::SpyglassPlugin;
use bevy::prelude::*;
use bevy::utils::HashMap;

fn main() {
    App::new()
        .register_type::<List>()
        .register_type::<Map>()
        .register_type::<Vec<i32>>()
        .register_type::<HashMap<String, i32>>()
        .add_plugins(DefaultPlugins)
        .add_plugin(SpyglassPlugin)
        .add_startup_system(setup)
        .run();
}

#[derive(Component, Default, Reflect)]
#[reflect(Component, Default)]
struct List(Vec<i32>);

#[derive(Component, Default, Reflect)]
#[reflect(Component, Default)]
struct Map(HashMap<String, i32>);

fn setup(mut c: Commands, q: Query<Entity>) {
    let window = q.single();
    c.entity(window).insert((
        List(vec![1, 2, 3]),
        Map(HashMap::from_iter([
            ("bruh".into(), 12),
            ("two".into(), 24),
        ])),
    ));
}

// fn setup(mut editors: ResMut<ValueEditors>) {
//     // editors.editors.insert(
//     //     std::any::type_name::<Window>().to_string(),
//     //     Box::new(|ui, value, _| {
//     //         ui.horizontal(|ui| {
//     //             ui.label("title: ");
//     //             let value = match value.reflect_mut() {
//     //                 bevy::reflect::ReflectMut::Struct(val) => val,
//     //                 _ => panic!(),
//     //             };
//     //             let title = value.get_field_mut::<String>("title").unwrap();

//     //             ui.text_edit_singleline(title);
//     //         });
//     //     }),
//     // );

//     // editors.applicators.insert(
//     //     std::any::type_name::<Window>().to_string(),
//     //     Box::new(|entity, value| {
//     //         let refl_comp = entity.world_scope(|world| {
//     //             world
//     //                 .resource::<AppTypeRegistry>()
//     //                 .read()
//     //                 .get_type_data::<ReflectComponent>(TypeId::of::<Window>())
//     //                 .unwrap()
//     //                 .clone()
//     //         });
//     //         refl_comp.apply(entity, &*value);
//     //     }),
//     // );
// }
