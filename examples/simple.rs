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
        .add_plugins(SpyglassPlugin)
        .add_systems(Startup, setup)
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
            ("test".into(), 12),
            ("two".into(), 24),
        ])),
    ));
}
