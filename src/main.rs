use bevy::{
    asset::RenderAssetUsages, color::palettes::css::WHITE, prelude::*, render::{
        render_resource::{AsBindGroup, Extent3d, TextureDimension, TextureFormat, TextureUsages},
        view::RenderLayers,
    }
};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            MaterialPlugin::<CloudMaterial>::default(),
            MaterialPlugin::<ScreenSpaceMaterial>::default(),
        ))
        .insert_resource(ClearColor(Color::srgb(0.5, 0.5, 0.9)))
        .add_systems(Startup, spawn_stuff)
        .add_systems(Update, (spin_camera, cloud_image_resize))
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
    mut screen_space_material: ResMut<Assets<ScreenSpaceMaterial>>,
    mut std_material: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    commands.spawn((
        Transform::from_translation(Vec3::splat(4.0)),
        Mesh3d(meshes.add(Sphere::new(0.2))),
        MeshMaterial3d(std_material.add(StandardMaterial::from_color(WHITE))),
    ));

    // image the cloud is rendered to
    let cloud_image_size = Extent3d {
        width: 192 * 2,
        height: 108 * 2,
        ..default()
    };
    let mut cloud_image = Image::new_fill(
        cloud_image_size.into(),
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::default(),
    );
    cloud_image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;
    let cloud_image_handle = images.add(cloud_image);
    commands.insert_resource(CloudImageHandle(cloud_image_handle.clone()));

    let first_pass_layer = RenderLayers::layer(1);

    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(1.2))),
        MeshMaterial3d(cloud_material.add(CloudMaterial {})),
        Transform::from_xyz(0.0, 1.5, 0.0),
        first_pass_layer.clone(),
    ));

    commands.spawn((
        Camera3d {
            ..Default::default()
        },
        Camera {
            target: cloud_image_handle.clone().into(),
            clear_color: Color::srgba(0.0, 0.0, 0.0, 0.0).into(),
            ..Default::default()
        },
        first_pass_layer,
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

    let screen_space_texture_handle = screen_space_material.add(ScreenSpaceMaterial {
        texture: cloud_image_handle,
    });
    commands.insert_resource(ScreenSpaceMaterialHandle(
        screen_space_texture_handle.clone(),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(1.2))),
        MeshMaterial3d(screen_space_texture_handle),
        Transform::from_xyz(0.0, 1.5, 0.0),
    ));

    commands.insert_resource(LastWindowSize(None));
}

#[derive(Clone, Asset, AsBindGroup, TypePath, Debug)]
struct CloudMaterial {}

impl Material for CloudMaterial {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        // "shaders/cloud.wgsl".into()
        "shaders/cloud.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

#[derive(Clone, Asset, AsBindGroup, TypePath, Debug)]
struct ScreenSpaceMaterial {
    #[texture(0)]
    #[sampler(1)]
    texture: Handle<Image>,
}

#[derive(Resource)]
struct ScreenSpaceMaterialHandle(Handle<ScreenSpaceMaterial>);

impl Material for ScreenSpaceMaterial {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "shaders/screen_space.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Premultiplied
    }
}

#[derive(Resource)]
struct CloudImageHandle(Handle<Image>);

#[derive(Resource)]
struct LastWindowSize(Option<UVec2>);

/// this is squared i.e., image_height = window_height/(2^CLOUD_DOWNSAMPLE)
const CLOUD_DOWNSAMPLE: u32 = 2;
fn cloud_image_resize(
    mut images: ResMut<Assets<Image>>,
    cloud_image_handle: ResMut<CloudImageHandle>,
    window: Query<&Window>,
    mut screen_space_material: ResMut<Assets<ScreenSpaceMaterial>>,
    screen_space_material_handle: ResMut<ScreenSpaceMaterialHandle>,
    mut last_window_size: ResMut<LastWindowSize>,
) {
    // return if we can't get a window or the window has the same size
    if let Ok(window) = window.single() {
        if let Some(dimension) = last_window_size.0 {
            if window.physical_size() == dimension {
                return;
            }
        }
        last_window_size.0 = Some(window.physical_size());
    } else {
        return;
    }

    if let Some(curr_cloud_image) = images.get_mut(cloud_image_handle.0.id())
        && let Ok(window) = window.single()
    {
        // image the cloud is rendered to
        let downsample_factor: u32 = (2u32).pow(CLOUD_DOWNSAMPLE);
        let cloud_image_size = Extent3d {
            width: window.physical_width() / downsample_factor,
            height: window.physical_height() / downsample_factor,
            ..default()
        };
        curr_cloud_image.resize(cloud_image_size);

        // yes, this should happen automatically - it's the same image handle
        // it doesn't
        screen_space_material.insert(
            screen_space_material_handle.0.id(),
            ScreenSpaceMaterial {
                texture: cloud_image_handle.0.clone(),
            },
        );
    }
}
