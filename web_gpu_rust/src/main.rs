use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use wgpu::util::DeviceExt;
use bytemuck::{Pod, Zeroable};

fn main() {
    pollster::block_on(run());
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct Uniforms {
    angle: f32,
}

async fn run() {

    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("Rotating Triangle")
        .build(&event_loop)
        .unwrap();

    let instance = wgpu::Instance::default();

    let surface = unsafe {
        instance.create_surface(&window)
    }.unwrap();

    let adapter = instance.request_adapter(
        &wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }
    ).await.unwrap();

    let (device, queue) = adapter.request_device(
        &wgpu::DeviceDescriptor::default(),
        None
    ).await.unwrap();

    let size = window.inner_size();

    let surface_format =
        surface.get_capabilities(&adapter).formats[0];

    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
        view_formats: vec![],
    };

    surface.configure(&device, &config);

    let vertices = [

        Vertex {
            position: [0.0, 0.5],
            color: [1.0, 0.0, 0.0],
        },

        Vertex {
            position: [-0.5, -0.5],
            color: [0.0, 1.0, 0.0],
        },

        Vertex {
            position: [0.5, -0.5],
            color: [0.0, 0.0, 1.0],
        },
    ];

    let vertex_buffer = device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        }
    );

    let mut uniforms = Uniforms { angle: 0.0 };

    let uniform_buffer = device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::bytes_of(&uniforms),
            usage:
                wgpu::BufferUsages::UNIFORM |
                wgpu::BufferUsages::COPY_DST,
        }
    );

    let bind_group_layout =
        device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {

                label: Some("Uniform Layout"),

                entries: &[
                    wgpu::BindGroupLayoutEntry {

                        binding: 0,

                        visibility:
                            wgpu::ShaderStages::VERTEX,

                        ty: wgpu::BindingType::Buffer {

                            ty: wgpu::BufferBindingType::Uniform,

                            has_dynamic_offset: false,

                            min_binding_size: None,
                        },

                        count: None,
                    }
                ],
            }
        );

    let bind_group = device.create_bind_group(
        &wgpu::BindGroupDescriptor {

            layout: &bind_group_layout,

            entries: &[
                wgpu::BindGroupEntry {

                    binding: 0,

                    resource:
                        uniform_buffer.as_entire_binding(),
                }
            ],

            label: Some("Bind Group"),
        }
    );

    let shader = device.create_shader_module(
        wgpu::ShaderModuleDescriptor {

            label: Some("Shader"),

            source: wgpu::ShaderSource::Wgsl(
                include_str!("shader.wgsl").into()
            ),
        }
    );

    let pipeline_layout =
        device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {

                label: Some("Pipeline Layout"),

                bind_group_layouts: &[&bind_group_layout],

                push_constant_ranges: &[],
            }
        );

    let render_pipeline =
        device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {

                label: Some("Render Pipeline"),

                layout: Some(&pipeline_layout),

                vertex: wgpu::VertexState {

                    module: &shader,

                    entry_point: "vs_main",

                    buffers: &[wgpu::VertexBufferLayout {

                        array_stride:
                            std::mem::size_of::<Vertex>()
                                as wgpu::BufferAddress,

                        step_mode:
                            wgpu::VertexStepMode::Vertex,

                        attributes: &[
                            wgpu::VertexAttribute {

                                offset: 0,

                                shader_location: 0,

                                format:
                                    wgpu::VertexFormat::Float32x2,
                            },

                            wgpu::VertexAttribute {

                                offset: 8,

                                shader_location: 1,

                                format:
                                    wgpu::VertexFormat::Float32x3,
                            },
                        ],
                    }],
                },

                fragment: Some(
                    wgpu::FragmentState {

                        module: &shader,

                        entry_point: "fs_main",

                        targets: &[Some(
                            wgpu::ColorTargetState {

                                format: surface_format,

                                blend: Some(
                                    wgpu::BlendState::REPLACE
                                ),

                                write_mask:
                                    wgpu::ColorWrites::ALL,
                            }
                        )],
                    }
                ),

                primitive: wgpu::PrimitiveState::default(),

                depth_stencil: None,

                multisample:
                    wgpu::MultisampleState::default(),

                multiview: None,
            }
        );

    event_loop.run(move |event, _, control_flow| {

        *control_flow = ControlFlow::Poll;

        match event {

            Event::RedrawRequested(_) => {

                uniforms.angle += 0.02;

                queue.write_buffer(
                    &uniform_buffer,
                    0,
                    bytemuck::bytes_of(&uniforms),
                );

                let frame =
                    surface.get_current_texture().unwrap();

                let view = frame.texture.create_view(
                    &wgpu::TextureViewDescriptor::default()
                );

                let mut encoder =
                    device.create_command_encoder(
                        &wgpu::CommandEncoderDescriptor {
                            label: Some("Render Encoder"),
                        }
                    );

                {
                    let mut rpass =
                        encoder.begin_render_pass(
                            &wgpu::RenderPassDescriptor {

                                label: Some("Render Pass"),

                                color_attachments: &[Some(
                                    wgpu::RenderPassColorAttachment {

                                        view: &view,

                                        resolve_target: None,

                                        ops: wgpu::Operations {

                                            load:
                                                wgpu::LoadOp::Clear(
                                                    wgpu::Color::BLACK
                                                ),

                                            store: true,
                                        },
                                    }
                                )],

                                depth_stencil_attachment: None,
                            }
                        );

                    rpass.set_pipeline(&render_pipeline);

                    rpass.set_bind_group(
                        0,
                        &bind_group,
                        &[]
                    );

                    rpass.set_vertex_buffer(
                        0,
                        vertex_buffer.slice(..)
                    );

                    rpass.draw(0..3, 0..1);
                }

                queue.submit(
                    std::iter::once(encoder.finish())
                );

                frame.present();
            }

            Event::MainEventsCleared => {
                window.request_redraw();
            }

            Event::WindowEvent {

                event: WindowEvent::CloseRequested,

                ..

            } => *control_flow = ControlFlow::Exit,

            _ => {}
        }
    });
}
