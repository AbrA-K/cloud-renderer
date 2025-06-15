use bevy::{
    color::palettes::css::WHITE, core_pipeline::prepass::DepthPrepass, pbr::NotShadowCaster,
    prelude::*, render::render_resource::AsBindGroup,
};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MaterialPlugin::<CloudMaterial>::default()))
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
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut cloud_material: ResMut<Assets<CloudMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // camera
    commands.spawn((
        Camera3d {
            ..Default::default()
        },
        Msaa::Off, // turn it off since it doesn't work on web
        SpinningCam {
            height: 3.0,
            distance: 5.0,
            speed: 0.5,
            sway_amount: 1.0,
            look_at: Vec3::new(0.0, 1.5, 0.0),
        },
        DepthPrepass,
    ));

    // circular base
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(6.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));

    // cloud
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(1.5))),
        MeshMaterial3d(cloud_material.add(CloudMaterial::new())),
        Transform::from_xyz(0.0, 1.5, 0.0),
        NotShadowCaster,
    ));

    // directional light
    commands.spawn((
        DirectionalLight {
            color: WHITE.into(),
            illuminance: 5000.0,
            shadows_enabled: true,
            ..Default::default()
        },
        Transform::default().with_rotation(Quat::from_euler(EulerRot::XYZ, -2.14, 0.5, 0.0)),
    ));
    // commands.spawn((
    //     Mesh3d(meshes.add(Cylinder::new(0.2, 5.0))),
    //     MeshMaterial3d(materials.add(Color::WHITE)),
    //     Transform::from_xyz(0.4, 0.2, 0.4),
    // ));
}

#[derive(Clone, Asset, AsBindGroup, TypePath, Debug)]
struct CloudMaterial {
    #[uniform(100)]
    color: Vec3,
}

impl CloudMaterial {
    fn new() -> Self {
        Self {
            color: Vec3::new(0.0, 1.0, 1.0),
        }
    }
}

impl Material for CloudMaterial {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "shaders/cloud.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}
