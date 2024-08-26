use bevy::prelude::*;
// use graph_node::GraphNodePlugin;
// mod graph_node;
mod material;
// mod mesh2d_manual;
// mod mesh2doriginal;
// mod pipeline;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: (500., 500.).into(),
                ..default()
            }),
            ..default()
        }))
        //.add_plugins(mesh2doriginal::ColoredMesh2dPlugin)
        .add_plugins(material::MaterialPlugin)
        .run();
}
