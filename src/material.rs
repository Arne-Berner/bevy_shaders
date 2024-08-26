use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, *},
    sprite::{Material2d, Material2dPlugin},
    window::{Window, WindowResized},
};

pub struct MaterialPlugin;

impl Plugin for MaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<YourShader2D>::default())
            .add_systems(Startup, setup)
            .add_systems(Update, size_quad.run_if(on_event::<WindowResized>()))
            .add_systems(Update, quit);
    }
}

// TODO put window dimensions in a central place
pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut your_shader: ResMut<Assets<YourShader2D>>,
    windows: Query<&Window>,
) {
    // 2D camera
    commands.spawn((Camera2dBundle { ..default() }, Cam2D));
    trace!("Spawned 2d Cam");

    let win = windows
        .get_single()
        .expect("Should be impossible to NOT get a window");
    let (width, height) = (win.width(), win.height());

    trace!("Set MaxSceenDims set to {width}, {height}");
    println!("width: {}, height: {}", width, height);

    // Quad
    commands.spawn((
        bevy::sprite::MaterialMesh2dBundle {
            // this gets resized to the right size
            mesh: meshes.add(Rectangle::new(1.0, 1.0)).into(),
            material: your_shader.add(YourShader2D {}),

            transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
            // .with_rotation(Quat::from_rotation_x(180.0)), // So that y is not upside down?
            ..default()
        },
        BillBoardQuad,
    ));
}

// TODO Add a texture
#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
// #[uuid = "f528511f-dcf2-4b0b-9522-a9df3a1a795b"]
pub struct YourShader2D {
    // could hold texture and mouse position
}

impl Material2d for YourShader2D {
    fn fragment_shader() -> ShaderRef {
        "material.wgsl".into()
    }
}

/// Resize the quad such that it's always the width/height of the viewport when in 2D mode.
pub fn size_quad(windows: Query<&Window>, mut query: Query<&mut Transform, With<BillBoardQuad>>) {
    let win = windows
        .get_single()
        .expect("Should be impossible to NOT get a window");

    let (width, height) = (win.width(), win.height());

    query.iter_mut().for_each(|mut transform| {
        transform.scale = Vec3::new(width * 0.95, height * 0.95, 1.0);
    });
}

/// Component: Marking the 2d geometry we use inplace of a custom vertex shader.
/// Used by: size_quad
#[derive(Component)]
pub struct BillBoardQuad;

/// Component: Marking the 2d camera.
/// Used by: the CamSwitch event.
#[derive(Component)]
pub struct Cam2D;

/// System:
/// Quits the app...
pub fn quit(input: Res<ButtonInput<KeyCode>>) {
    if input.just_pressed(KeyCode::Escape) {
        std::process::exit(0)
    }
}
