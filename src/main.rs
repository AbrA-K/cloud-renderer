use bevy::{
    color::palettes::css::WHITE,
    core_pipeline::prepass::DepthPrepass,
    pbr::NotShadowCaster,
    prelude::*,
    render::{render_resource::AsBindGroup, storage::ShaderStorageBuffer},
};
use rand::Rng;

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
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut cloud_material: ResMut<Assets<CloudMaterial>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    asset_server: Res<AssetServer>,
) {
    // camera
    commands.spawn((
        Camera3d {
            ..Default::default()
        },
        Msaa::Off, // turn it off since it doesn't work on web
        SpinningCam {
            height: 1.5,
            distance: 3.0,
            speed: 0.5,
            sway_amount: 0.2,
            look_at: Vec3::new(0.0, 1.5, 0.0),
        },
        DepthPrepass,
    ));

    // circular base
    // commands.spawn((
    //     Mesh3d(meshes.add(Circle::new(6.0))),
    //     MeshMaterial3d(materials.add(Color::WHITE)),
    //     Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    // ));

    // cloud
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(1.5))),
        MeshMaterial3d(cloud_material.add(CloudMaterial::new(buffers))),
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
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(1.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_xyz(4.0, 4.0, 4.0),
    ));
}

#[derive(Clone, Asset, AsBindGroup, TypePath, Debug)]
struct CloudMaterial {
    #[uniform(100)]
    color: Vec3,
    #[storage(0, read_only)]
    offsets: Handle<ShaderStorageBuffer>,
}

impl CloudMaterial {
    fn new(mut buffers: ResMut<Assets<ShaderStorageBuffer>>) -> Self {
        Self {
            color: Vec3::new(0.0, 1.0, 1.0),
	    offsets: buffers.add(get_worley_world()),
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

const WORLEY_WORLD_SIZE: usize = 10;
type WORLEY_OFFSET_ARRAY = [[[f32; WORLEY_WORLD_SIZE]; WORLEY_WORLD_SIZE]; WORLEY_WORLD_SIZE];
fn get_worley_world() -> WORLEY_OFFSET_ARRAY {
    use rand;
    let mut rng = rand::rng();
    let mut world = [[[0.0; WORLEY_WORLD_SIZE]; WORLEY_WORLD_SIZE]; WORLEY_WORLD_SIZE];
    for slice in world.iter_mut() {
        for line in slice.iter_mut() {
            for cell in line.iter_mut() {
                *cell = rng.random_range(0.0..1.0);
            }
        }
    }
    return world;
}
