use std::borrow::Cow;

use bevy::{
    prelude::{App, AssetPlugin, PluginGroup, Commands, ResMut, Assets, Image, Plugin, Handle, Resource, Deref, Res, FromWorld, World, AssetServer, Vec2, Camera2dBundle, IntoSystemConfig},
    utils::default,
    DefaultPlugins, render::{render_resource::{Extent3d, BindGroupLayout, CachedComputePipelineId, BindGroupDescriptor, BindGroupEntry, BindingResource, BindGroup, BindGroupLayoutDescriptor, BindGroupLayoutEntry, ShaderStages, BindingType, StorageTextureAccess, TextureFormat, TextureViewDimension, PipelineCache, ComputePipelineDescriptor, TextureDimension, TextureUsages, CachedPipelineState, ComputePassDescriptor}, extract_resource::{ExtractResourcePlugin, ExtractResource}, RenderApp, render_asset::RenderAssets, renderer::{RenderDevice, RenderContext}, RenderSet, render_graph::{RenderGraph, self}}, sprite::{SpriteBundle, Sprite},
};
use bevy_rust_gpu::{RustGpuPlugin, RustGpuBuilderOutput};

/// Workspace-relative path to SPIR-V shader
const SHADER_PATH: &'static str = "rust-gpu/shader.rust-gpu.msgpack";

// const ENTRY_POINTS_PATH: &'static str = "crates/viewer/entry_points.json";

// const ENTRY_POINT: &str = "compute_primes";

const SIZE: (u32, u32) = (1280, 720);
const WORKGROUP_SIZE: u32 = 1;


fn main() {
    let mut app = App::default();

    // Add default plugins
    app.add_plugins(DefaultPlugins.set(
        // Configure the asset plugin to watch the workspace path for changes
        AssetPlugin {
            watch_for_changes: true,
            ..default()
        },
    ));

    // Add the Rust-GPU plugin
    app.add_plugin(RustGpuPlugin::default());

    app.add_plugin(PrimeTexturePlugin)
        .add_startup_system(setup)
        .run();

    // Setup scene
    app.add_startup_system(setup);

    // Run
    app.run();
}

struct PrimeTexturePlugin;

#[derive(Resource, Clone, Deref, ExtractResource)]
struct PrimeTextureImage(Handle<Image>);

#[derive(Resource)]
struct GameOfLifeImageBindGroup(BindGroup);



#[derive(Resource)]
pub struct GameOfLifePipeline {
    texture_bind_group_layout: BindGroupLayout,
    init_pipeline: CachedComputePipelineId
}

impl Plugin for PrimeTexturePlugin {
    fn build(&self, app: &mut App) {
        // Extract the game of life image resource from the main world into the render world
        // for operation on by the compute shader and display on the sprite.
        app.add_plugin(ExtractResourcePlugin::<PrimeTextureImage>::default());
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<GameOfLifePipeline>()
            .add_system(queue_bind_group.in_set(RenderSet::Queue));

        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        render_graph.add_node("game_of_life", GameOfLifeNode::default());
        render_graph.add_node_edge(
            "game_of_life",
            bevy::render::main_graph::node::CAMERA_DRIVER,
        );
    }
}


fn queue_bind_group(
    mut commands: Commands,
    pipeline: Res<GameOfLifePipeline>,
    gpu_images: Res<RenderAssets<Image>>,
    game_of_life_image: Res<PrimeTextureImage>,
    render_device: Res<RenderDevice>,
) {
    let view = &gpu_images[&game_of_life_image.0];
    let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &pipeline.texture_bind_group_layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: BindingResource::TextureView(&view.texture_view),
        }],
    });
    commands.insert_resource(GameOfLifeImageBindGroup(bind_group));
}

impl FromWorld for GameOfLifePipeline {
    fn from_world(world: &mut World) -> Self {
        let texture_bind_group_layout =
            world
                .resource::<RenderDevice>()
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::ReadWrite,
                            format: TextureFormat::Rgba8Unorm,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    }],
                });
        let shader = world
            .resource::<AssetServer>()
            .load::<RustGpuBuilderOutput, _>(SHADER_PATH);

        let pipeline_cache = world.resource::<PipelineCache>();
        let init_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![texture_bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: todo!("How do I generate bevy shader from the RustGpuBuilderOutput {shader:?}?"),
            shader_defs: vec![],
            entry_point: Cow::from("compute_primes"),
        });


        GameOfLifePipeline {
            texture_bind_group_layout,
            init_pipeline,
        }
    }
}


fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let mut image = Image::new_fill(
        Extent3d {
            width: SIZE.0,
            height: SIZE.1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Rgba8Unorm,
    );
    image.texture_descriptor.usage =
        TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;
    let image = images.add(image);

    commands.spawn(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(SIZE.0 as f32, SIZE.1 as f32)),
            ..default()
        },
        texture: image.clone(),
        ..default()
    });
    commands.spawn(Camera2dBundle::default());

    commands.insert_resource(PrimeTextureImage(image));
}



enum GameOfLifeState {
    Loading,
    Init,
    Update,
}

struct GameOfLifeNode {
    state: GameOfLifeState,
}

impl Default for GameOfLifeNode {
    fn default() -> Self {
        Self {
            state: GameOfLifeState::Loading,
        }
    }
}

impl render_graph::Node for GameOfLifeNode {
    fn update(&mut self, world: &mut World) {
        let pipeline = world.resource::<GameOfLifePipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        // if the corresponding pipeline has loaded, transition to the next stage
        match self.state {
            GameOfLifeState::Loading => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.init_pipeline)
                {
                    self.state = GameOfLifeState::Init;
                }
            }
            // FIXME(samoylovfp): there is only one pipeline
            GameOfLifeState::Init => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.init_pipeline)
                {
                    self.state = GameOfLifeState::Update;
                }
            }
            GameOfLifeState::Update => {}
        }
    }

    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let texture_bind_group = &world.resource::<GameOfLifeImageBindGroup>().0;
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<GameOfLifePipeline>();

        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor::default());

        pass.set_bind_group(0, texture_bind_group, &[]);

        // select the pipeline based on the current state
        match self.state {
            GameOfLifeState::Loading => {}
            GameOfLifeState::Init => {
                let init_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.init_pipeline)
                    .unwrap();
                pass.set_pipeline(init_pipeline);
                pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 / WORKGROUP_SIZE, 1);
            }
            // FIXME(samoylovfp): there is only one pipeline
            GameOfLifeState::Update => {
                let update_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.init_pipeline)
                    .unwrap();
                pass.set_pipeline(update_pipeline);
                pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 / WORKGROUP_SIZE, 1);
            }
        }

        Ok(())
    }
}