use std::{borrow::Cow, sync::Arc};

use glam::Vec3;
use wgpu::{util::DeviceExt, Adapter, BindGroup, BindGroupLayout, Buffer, Color, CommandEncoderDescriptor, Device, DeviceDescriptor, Features, FragmentState, Instance, Limits, LoadOp, MemoryHints, Operations, PipelineLayout, PipelineLayoutDescriptor, PowerPreference, Queue, RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions, ShaderModuleDescriptor, ShaderSource, StoreOp, Surface, SurfaceConfiguration, TextureFormat, TextureView, TextureViewDescriptor, Trace, VertexState};
use winit::{dpi::{PhysicalPosition, PhysicalSize}, event::DeviceEvent, event_loop::EventLoopProxy, window::Window};

use crate::{camera::{controller::CameraController, CameraUniform, OrbitCamera}, light::LightUniform, texture::Texture, vertex::Vertex};

/// The number of samples taken when using multisample anti-aliasing.
/// Valid values are `1` (no MSAA) or `4`.
// #[cfg(feature = "msaa")]
// const MSAA_SAMPLE_COUNT: u32 = 4;
// #[cfg(not(feature = "msaa"))]
const MSAA_SAMPLE_COUNT: u32 = 1;



fn create_camera(size: PhysicalSize<u32>) -> (OrbitCamera, CameraController, CameraUniform) {
    let mut camera = OrbitCamera::new(
        15.0,
        0.0,
        0.0,
        Vec3::new(0.0, 0.0, 0.0),
        size.width as f32 / size.height as f32,
    );
    camera.bounds.min_distance = Some(10.0);
    let camera_controller = CameraController::new(0.002, 0.5);

    let mut camera_uniform = CameraUniform::default();
    camera_uniform.update_view_proj(&camera);

    (camera, camera_controller, camera_uniform)
}

fn create_camera_bind_group(device: &Device, camera_uniform: &CameraUniform) -> (BindGroup, BindGroupLayout, Buffer) {
    let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Camera Buffer"),
        contents: bytemuck::cast_slice(&[*camera_uniform]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });


    let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

    let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &camera_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: camera_buffer.as_entire_binding(),
        }],
        label: Some("camera_bind_group"),
    });

    (camera_bind_group, camera_bind_group_layout, camera_buffer)

}

fn create_light(device: &Device) -> (BindGroup, BindGroupLayout, Buffer, LightUniform) {
    let light_uniform = LightUniform {
        position: [2.0, 6.0, 4.0, 1.0],
        color: [1.0, 1.0, 1.0, 0.1],
    };
    let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Light VB"),
        contents: bytemuck::cast_slice(&[light_uniform]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });
    let light_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: None,
        }
    );
    let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &light_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: light_buffer.as_entire_binding(),
        }],
        label: None,
    });

    (light_bind_group, light_bind_group_layout, light_buffer, light_uniform)
}

fn create_texture(device: &Device, queue: &Queue, surface_config: &SurfaceConfiguration) -> (TextureView, BindGroup, BindGroupLayout, Texture) {
    // let diffuse_bytes = include_bytes!("assets/test.png");
    let diffuse_texture = Texture::from_bytes(&device, &queue, &[], "grid.png").unwrap();

    let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

    let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &texture_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
            },
        ],
        label: Some("diffuse_bind_group"),
    });

    let depth_texture_view = Texture::create_depth_texture(
        &device,
        &surface_config,
        MSAA_SAMPLE_COUNT,
        "depth_texture",
    );

    (depth_texture_view, diffuse_bind_group, texture_bind_group_layout, diffuse_texture)

}

fn create_verts(device: &Device) -> (Buffer, Buffer, u32) {
    let (vertices, indices) = crate::sphere::get_sphere_vertices(10.0);

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(&indices),
        usage: wgpu::BufferUsages::INDEX,
    });
    let num_indices = indices.len() as u32;

    (vertex_buffer, index_buffer, num_indices)
}

pub async fn create_graphics(window: Arc<Window>, proxy: EventLoopProxy<Graphics>) {
    let instance = Instance::default();
    let surface = instance.create_surface(Arc::clone(&window)).unwrap();
    let adapter = instance
        .request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::default(), // Power preference for the device
            force_fallback_adapter: false, // Indicates that only a fallback ("software") adapter can be used
            compatible_surface: Some(&surface), // Guarantee that the adapter can render to this surface
        })
        .await
        .expect("Could not get an adapter (GPU).");

    let (device, queue) = adapter
        .request_device(
            &DeviceDescriptor {
                label: None,
                required_features: Features::empty(), // Specifies the required features by the device request. Fails if the adapter can't provide them.
                required_limits: Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
                memory_hints: MemoryHints::Performance,
                trace: Trace::Off
            }
        )
        .await
        .expect("Failed to get device");

    // Get physical pixel dimensiosn inside the window
    let size = window.inner_size();
    // Make the dimensions at least size 1, otherwise wgpu would panic
    let width = size.width.max(1);
    let height = size.height.max(1);

    let surface_config = surface.get_default_config(&adapter, width, height).unwrap();
    surface.configure(&device, &surface_config);

    // Get textures
    let (depth_texture_view, diffuse_bind_group, texture_bind_group_layout, diffuse_texture) = create_texture(&device, &queue, &surface_config);

    // Get camera
    let (camera, camera_controller, camera_uniform) = create_camera(size);
    let (camera_bind_group,camera_bind_group_layout, camera_buffer) = create_camera_bind_group(&device, &camera_uniform);

    // Get light
    let (light_bind_group, light_bind_group_layout, light_buffer, light_uniform) = create_light(&device);


    let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[
            &texture_bind_group_layout,
            &camera_bind_group_layout,
            &light_bind_group_layout,
        ],
        push_constant_ranges: &[],
    });

    let multisampled_framebuffer = Texture::create_multisampled_framebuffer(
        &device,
        &surface_config,
        MSAA_SAMPLE_COUNT,
        "multisampled_framebuffer",
    );

    let render_pipeline = create_pipeline(&device, render_pipeline_layout,surface_config.format);

    let (vertex_buffer, index_buffer, num_indices) = create_verts(&device);

    let gfx = Graphics {
        window: window.clone(),
        instance,
        cursor_pos: PhysicalPosition::new(0.0, 0.0),
        surface,
        surface_config,
        adapter,
        device,
        queue,
        render_pipeline,

        vertex_buffer,
        index_buffer,
        num_indices,

        multisampled_framebuffer,
        depth_texture_view,
        diffuse_texture,
        diffuse_bind_group,

        camera,
        camera_buffer,
        camera_bind_group,
        camera_controller,
        camera_uniform,

        light_uniform,
        light_buffer,
        light_bind_group,
    };

    let _ = proxy.send_event(gfx);
}


fn create_pipeline(device: &Device, layout: PipelineLayout, swap_chain_format: TextureFormat) -> RenderPipeline {
    let shader = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("Shader"),
        source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("assets/shader.wgsl"))),
    });

    device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&layout),
        vertex: VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[Vertex::desc()],
            compilation_options: Default::default(),
        },
        fragment: Some(FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(swap_chain_format.into())],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            // Requires Features::NON_FILL_POLYGON_MODE
            polygon_mode: wgpu::PolygonMode::Fill,
            // Requires Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: MSAA_SAMPLE_COUNT,
            ..Default::default()
        },
        multiview: None,
        cache: None,
    })
}


#[derive(Debug)]
pub struct Graphics {
    window: Arc<Window>,
    instance: Instance,
    cursor_pos: PhysicalPosition<f32>,
    surface: Surface<'static>,
    surface_config: SurfaceConfiguration,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    render_pipeline: RenderPipeline,
    // Triangles
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    num_indices: u32,
    // Texture Stuff
    multisampled_framebuffer: TextureView,
    depth_texture_view: TextureView,
    diffuse_texture: Texture,
    diffuse_bind_group: BindGroup,
    // The camera used for rendering the scene.
    camera: OrbitCamera,
    camera_buffer: Buffer,
    camera_bind_group: BindGroup,
    pub camera_controller: CameraController,
    camera_uniform: CameraUniform,
    // Lighting Stuff
    light_uniform: LightUniform,
    light_buffer: Buffer,
    light_bind_group: BindGroup,
}

impl Graphics {
    /// Resizes the renderer and adjusts the camera aspect.
    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.surface_config.width = new_size.width.max(1);
        self.surface_config.height = new_size.height.max(1);

        self.depth_texture_view = Texture::create_depth_texture(
            &self.device,
            &self.surface_config,
            MSAA_SAMPLE_COUNT,
            "depth_texture",
        );
        self.multisampled_framebuffer = Texture::create_multisampled_framebuffer(
            &self.device,
            &self.surface_config,
            MSAA_SAMPLE_COUNT,
            "multisampled_framebuffer",
        );

        self.surface.configure(&self.device, &self.surface_config);

        self.camera.aspect = self.surface_config.width as f32 / self.surface_config.height as f32;
    }

    /// Updates the state.
    pub fn update(&mut self) {
        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );

        // Update the light so that it is transformed with the camera
        self.light_uniform.position = [
            self.camera_uniform.view_position[0],
            self.camera_uniform.view_position[1],
            self.camera_uniform.view_position[2],
            1.0,
        ];
        self.queue.write_buffer(
            &self.light_buffer,
            0,
            bytemuck::cast_slice(&[self.light_uniform]),
        );
    }

    pub fn update_cursor_position(&mut self, pos: PhysicalPosition<f64>) {
        self.cursor_pos = pos.cast();
        println!("{:?}", self.cursor_pos);
    }

    /// Renders the scene based on the [State].
    pub fn draw(&mut self) {
        let frame = self.surface.get_current_texture().expect("Failed to acquire next swap chain texture.");
        let view = frame.texture.create_view(&TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor { label: Some("Render Encoder") });

        let rpass_view = if MSAA_SAMPLE_COUNT == 1 {
            &view
        } else {
            &self.multisampled_framebuffer
        };

        {
            let mut r_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: rpass_view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color { r: 0.1, g: 0.2, b: 0.3, a: 1.0 }),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &self.depth_texture_view,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            r_pass.set_pipeline(&self.render_pipeline);
            r_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            r_pass.set_bind_group(1, &self.camera_bind_group, &[]);
            r_pass.set_bind_group(2, &self.light_bind_group, &[]);

            r_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            r_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            r_pass.draw_indexed(0..self.num_indices, 0, 0..1);

        } // `r_pass` dropped here

        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }

    pub fn process_camera_event(&mut self,  event: &DeviceEvent) {
        self.camera_controller.process_events(event, &self.window, &mut self.camera);
    }
}