use bevy::{
    pbr::NotShadowCaster,
    prelude::*,
    render::render_resource::AsBindGroup,
};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MaterialPlugin::<CloudMaterial>::default()))
        .insert_resource(ClearColor(Color::srgb(0.5, 0.5, 0.9)))
        .add_systems(Startup, spawn_stuff)
        .add_systems(Update, spin_camera)
        .run();
}

#[derive(Component)]
struct SpinningCam {
    height: f32,
    distance: f32,
    speed: f32,
    sway_amount: f32,
    look_at: Vec3,
}

fn spin_camera(mut cams: Query<(&mut Transform, &SpinningCam)>, time: Res<Time>) {
    cams.iter_mut()
        .for_each(|(mut transform, spinning_cam_vars)| {
            let new_z =
                (time.elapsed_secs() * spinning_cam_vars.speed).cos() * spinning_cam_vars.distance;
            let new_x =
                (time.elapsed_secs() * spinning_cam_vars.speed).sin() * spinning_cam_vars.distance;
            let sway_y = (time.elapsed_secs() * spinning_cam_vars.speed / 0.35).sin()
                * spinning_cam_vars.sway_amount;
            let new_transform =
                Transform::from_xyz(new_x, spinning_cam_vars.height + sway_y, new_z)
                    .looking_at(spinning_cam_vars.look_at, Vec3::Y);
            *transform = new_transform;
        });
}

fn spawn_stuff(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut cloud_material: ResMut<Assets<CloudMaterial>>,
) {
    // camera
    commands.spawn((
        Camera3d {
            ..Default::default()
        },
        // turn it off since it doesn't work on web
        Msaa::Off,
        SpinningCam {
            height: 1.5,
            distance: 3.0,
            speed: 0.5,
            sway_amount: 0.2,
            look_at: Vec3::new(0.0, 1.5, 0.0),
        },
    ));

    // cloud
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(1.5))),
        MeshMaterial3d(cloud_material.add(CloudMaterial {})),
        Transform::from_xyz(0.0, 1.5, 0.0),
        NotShadowCaster,
    ));
}

#[derive(Clone, Asset, AsBindGroup, TypePath, Debug)]
struct CloudMaterial {
}

impl Material for CloudMaterial {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "shaders/cloud.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}
