use bevy::{
    asset::load_internal_asset,
    prelude::*,
    render::{
        graph::CameraDriverLabel,
        mesh::PrimitiveTopology,
        render_graph::{Node, NodeRunError, RenderGraph, RenderGraphContext, RenderLabel},
        render_phase::TrackedRenderPass,
        render_resource::{
            BindGroupEntry, BindGroupLayout, BindGroupLayoutEntry, BindingResource, BindingType,
            BlendState, BufferUsages, CachedRenderPipelineId, ColorTargetState, ColorWrites,
            CommandEncoderDescriptor, Face, FragmentState, FrontFace, ImageCopyTexture,
            ImageDataLayout, IndexFormat, LoadOp, MultisampleState, Operations, Origin3d,
            PipelineCache, PolygonMode, PrimitiveState, RawBufferVec, RenderPassColorAttachment,
            RenderPassDescriptor, RenderPipelineDescriptor, SamplerBindingType, ShaderStages,
            StoreOp, TextureAspect, TextureFormat, TextureSampleType, TextureViewDescriptor,
            TextureViewDimension, VertexAttribute, VertexBufferLayout, VertexFormat, VertexState,
            VertexStepMode,
        },
        renderer::{RenderContext, RenderDevice, RenderQueue},
        texture::{ImageSampler, ImageSamplerDescriptor},
        view::ExtractedWindows,
        Extract, RenderApp,
    },
};
// use encase::ShaderType; instead
use bytemuck::{Pod, Zeroable};

// You can easily organize your code with plugins in bevy, so that is what we are doing here.
// As a plugin we can interact with App to get everything we need.
pub struct Textures;

// This together with load_internal_asset allows us to include the shader at compile time.
// That way we do not have to interface with bevy_asset and we can be sure the shader is available immediately.
pub const SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(307062806978783518533214479195188549290);

impl Plugin for Textures {
    fn build(&self, app: &mut App) {
        load_internal_asset!(app, SHADER_HANDLE, "shader.wgsl", Shader::from_wgsl);
        app.init_resource::<MyTexture>();

        // Bevy splits up your usual logic world and the render world to allow for things such as pipelined rendering.
        // Later tutorials will go into more depth on how those two interact with each other.
        // For now everything we care about happens on the RenderApp and we do not need anything from the usual logic world.
        let render_app = match app.get_sub_app_mut(RenderApp) {
            Some(render_app) => render_app,
            None => return,
        };

        render_app.init_resource::<MyExtractedTexture>();
        render_app.add_systems(ExtractSchedule, extract_my_texture);

        // In wgpu we need to set up some stuff so the RenderPassDescriptor that we create is actually "run".
        // Bevy has built RenderGraphs on top. They are acyclic graphs that define Nodes that can connect to other Nodes.
        // A node can have inputs and outputs - for our usecase we will have a node without any of those.
        // Within a node you will usually create your RenderPassDescriptor, but you can also do other stuff here.
        // Bevy takes care of running every node in the RenderGraph.
        // In the future we will interface with RenderGraph to add sub graphs, that are driven by Cameras.
        let main_pass_node = MainPassNode;
        let mut render_graph = render_app.world_mut().resource_mut::<RenderGraph>();
        render_graph.add_node(MainPassNodeLabel, main_pass_node);
        // It is necessary to ensure our Node is run after the CameraDriverNode, because it may also write to the view texture.
        render_graph.add_node_edge(CameraDriverLabel, MainPassNodeLabel);
    }

    fn finish(&self, app: &mut App) {
        let render_app = match app.get_sub_app_mut(RenderApp) {
            Some(render_app) => render_app,
            None => return,
        };

        // This needs access to RenderDevice at creation time, so it needs to be done in finish.
        render_app.init_resource::<MainPipeline>();
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct MainPassNodeLabel;

#[derive(Default)]
pub struct MainPassNode;
impl Node for MainPassNode {
    fn run<'w>(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext<'w>,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        // The only thing we need right now is a texture view to write to.
        // Whereas wgpu will have preivously will previously interfaces with a windowing library to retrieve the Surface, we won't have to do any of that because bevy already did it.
        // The MainApp has extracted any windows to the RenderApp for us to use. The concept of extraction will be explored further in a future tutorial.
        let extracted_windows = world.resource::<ExtractedWindows>();
        let primary_window_entity = extracted_windows.primary.unwrap();
        let Some(view) = extracted_windows
            .get(&primary_window_entity)
            .and_then(|extracted_window| extracted_window.swap_chain_texture_view.as_ref())
        else {
            return Ok(());
        };

        // Image stuff
        let extracted_image = world.resource::<MyExtractedTexture>().0.clone();

        let render_device = world.resource::<RenderDevice>();
        let diffuse_texture = render_device.create_texture(&extracted_image.texture_descriptor);

        let diffuse_texture_view = diffuse_texture.create_view(
            &extracted_image
                .texture_view_descriptor
                .as_ref()
                .unwrap_or(&TextureViewDescriptor::default()),
        );
        let diffuse_sampler_descriptor = match &extracted_image.sampler {
            ImageSampler::Default => &ImageSamplerDescriptor::linear(), // TODO: workaround to do the same thing as ImagePlugin
            ImageSampler::Descriptor(descriptor) => descriptor,
        };
        let diffuse_sampler = render_device.create_sampler(&diffuse_sampler_descriptor.as_wgpu());
        let image_data = extracted_image.data.clone();

        let texture_size = extracted_image.texture_descriptor.size;

        let render_queue = world.resource::<RenderQueue>();
        render_queue.write_texture(
            // Tells wgpu where to copy the pixel data
            ImageCopyTexture {
                texture: &diffuse_texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            // The actual pixel data
            &image_data,
            // The layout of the texture
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * texture_size.width),
                rows_per_image: Some(texture_size.height),
            },
            texture_size,
        );

        // RenderContext will give us access to everything that the wgpu tutorial uses, so the following code is almost the same.
        render_context.add_command_buffer_generation_task(move |render_device| {
            // Command encoder setup
            let mut command_encoder =
                render_device.create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

            let render_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(
                            LinearRgba {
                                red: 0.1,
                                green: 0.2,
                                blue: 0.3,
                                alpha: 1.0,
                            }
                            .into(),
                        ),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // Tracked Render Pass is not stricly necessary, but the more idiomatic choice
            let mut tracked_render_pass = TrackedRenderPass::new(&render_device, render_pass);
            // We have previously created a RenderPipeline in MainPipeline via PipelineCache, so we need to retrieve it here.
            // Since pipeline creaton via PipelineCache is async, it might not be available from the very start.
            let main_pipeline = world.resource::<MainPipeline>();
            let texture_bind_group_layout = &main_pipeline.texture_bind_group_layout;
            let diffuse_bind_group = render_device.create_bind_group(
                Some("diffuse_bind_group"),
                texture_bind_group_layout,
                &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&diffuse_texture_view),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(&diffuse_sampler),
                    },
                ],
            );
            let pipeline_cache = world.resource::<PipelineCache>();
            if let Some(pipeline) = pipeline_cache.get_render_pipeline(main_pipeline.pipeline) {
                tracked_render_pass.set_render_pipeline(pipeline);

                // Lifetime issue with diffuse_bind_group
                tracked_render_pass.set_bind_group(0, &diffuse_bind_group, &[]);

                let vertices = &main_pipeline.vertices;
                let indices = &main_pipeline.indices;
                tracked_render_pass.set_vertex_buffer(0, vertices.buffer().unwrap().slice(..));
                tracked_render_pass.set_index_buffer(
                    indices.buffer().unwrap().slice(..),
                    0,
                    IndexFormat::Uint16,
                );
                tracked_render_pass.draw_indexed(0..(indices.len() as u32) - 1, 0, 0..1);
                // TODO: -1, because extra u16 at the end for padding
            }
            drop(tracked_render_pass);

            // No need to submit the command to the queue, because bevy will first batch them and queue them all together.
            // Bevy will also take care of presenting to the texture.
            command_encoder.finish()
        });

        Ok(())
    }
}

#[derive(Resource)]
pub struct MainPipeline {
    pub pipeline: CachedRenderPipelineId,
    pub vertices: RawBufferVec<Vertex>,
    pub indices: RawBufferVec<u16>,
    pub texture_bind_group_layout: BindGroupLayout,
}

// From world allows automatic initialization at creation time with the whole world available.
impl FromWorld for MainPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let render_queue = world.resource::<RenderQueue>();

        // Texture bind group
        let texture_bind_group_layout = render_device.create_bind_group_layout(
            Some("texture_bind_group_layout"),
            &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                        sample_type: TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    // This should match the filterable field of the
                    // corresponding Texture entry above.
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        );

        let render_pipeline_descriptor = RenderPipelineDescriptor {
            label: Some("Render Pipeline Layout".into()),
            layout: vec![texture_bind_group_layout.clone()],
            push_constant_ranges: vec![],
            vertex: VertexState {
                shader: SHADER_HANDLE,
                shader_defs: vec![],
                entry_point: "vs_main".into(),
                buffers: vec![Vertex::desc()],
            },
            fragment: Some(FragmentState {
                shader: SHADER_HANDLE,
                shader_defs: vec![],
                entry_point: "fs_main".into(),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::Bgra8UnormSrgb, // TODO: This needs to be the same as SurfaceTexture, but I can not retrieve it at this point
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        };

        // Vertex and index buffers
        let mut vbo = RawBufferVec::new(BufferUsages::VERTEX);
        let mut ibo = RawBufferVec::new(BufferUsages::INDEX);
        for vertex in VERTICES {
            vbo.push(*vertex);
        }
        for index in INDICES {
            ibo.push(*index);
        }
        vbo.write_buffer(render_device, render_queue);
        ibo.write_buffer(render_device, render_queue);

        // This takes care of asynchronously creating the render pipeline from the descriptor.
        let pipeline_cache = world.resource_mut::<PipelineCache>();
        Self {
            pipeline: pipeline_cache.queue_render_pipeline(render_pipeline_descriptor),
            vertices: vbo,
            indices: ibo,
            texture_bind_group_layout,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.0868241, 0.49240386, 0.0],
        tex_coords: [0.4131759, 0.99240386],
    }, // A
    Vertex {
        position: [-0.49513406, 0.06958647, 0.0],
        tex_coords: [0.0048659444, 0.56958647],
    }, // B
    Vertex {
        position: [-0.21918549, -0.44939706, 0.0],
        tex_coords: [0.28081453, 0.05060294],
    }, // C
    Vertex {
        position: [0.35966998, -0.3473291, 0.0],
        tex_coords: [0.85967, 0.1526709],
    }, // D
    Vertex {
        position: [0.44147372, 0.2347359, 0.0],
        tex_coords: [0.9414737, 0.7347359],
    }, // E
];

const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4, 0]; // TODO: This needed padding to be 4Byte aligned

impl Vertex {
    fn desc() -> VertexBufferLayout {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as u64,
            step_mode: VertexStepMode::Vertex,
            attributes: vec![
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x3,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as u64,
                    shader_location: 1,
                    format: VertexFormat::Float32x2,
                },
            ],
        }
    }
}

// We will make use of bevy asset loading
#[derive(Debug, Resource)]
pub struct MyTexture(pub Handle<Image>);

impl FromWorld for MyTexture {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        println!("Loading image");
        Self(asset_server.load("bevy+wgpu.png"))
    }
}

#[derive(Debug, Default, Resource)]
pub struct MyExtractedTexture(pub Image);

// TODO: Change to asset events
pub fn extract_my_texture(
    my_texture: Extract<Res<MyTexture>>,
    images: Extract<Res<Assets<Image>>>,
    mut my_extracted_texture: ResMut<MyExtractedTexture>,
) {
    if let Some(image) = images.get(&my_texture.0) {
        println!("Extracting image");
        my_extracted_texture.0 = image.clone();
    }
}
