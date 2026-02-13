use std::collections::{HashMap, HashSet};

use bevy_ecs::prelude::*;
use glam::Vec2;
use wgpu::util::DeviceExt;
use winit::{event::WindowEvent, keyboard::KeyCode};

use crate::{
    app::{self, BevyApp},
    delta_time::DeltaTime,
    render_resources::RenderQueue,
    winnit::{WinitDeviceEvent, WinitWindowEvent},
};

pub fn initialize(app: &mut BevyApp) {
    app.world.get_resource_or_init::<Schedules>().add_systems(
        crate::schedule::Update,
        (
            camera_system.after(camera_buffer_system),
            camera_buffer_system,
        ),
    );
}

fn camera_buffer_system(cameras: Query<&mut Camera>, queue: Res<RenderQueue>) {
    for mut camera in cameras {
        camera.update(&queue.0);
    }
}

fn camera_system(
    mut de_reader: MessageReader<WinitDeviceEvent>,
    mut we_reader: MessageReader<WinitWindowEvent>,
    mut camera: Query<&mut Camera>,
    mut keys_pressed: Local<HashSet<KeyCode>>,
    dt: Res<DeltaTime>,
) {
    // DANGER: This is super sketch and will break the moment i try to do anything else with
    // multiple cameras, or read any other kind of input (such as for resizing) yay!
    // TODO: DO THIS PROPERLY, HAVE A WINIT EVENT -> ENGINE EVENT mapping system.

    let Ok(mut camera) = camera.single_mut() else {
        return;
    };

    for WinitDeviceEvent(e) in de_reader.read() {
        match e {
            winit::event::DeviceEvent::MouseMotion { delta } => {
                const MOUSE_SENSITIVITY: f32 = 0.001;
                camera.rotate(Vec2::new(delta.0 as f32, delta.1 as f32) * MOUSE_SENSITIVITY);
            }
            _ => {}
        }
    }
    for WinitWindowEvent(e) in we_reader.read() {
        match e {
            winit::event::WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => {
                let winit::keyboard::PhysicalKey::Code(key) = event.physical_key else {
                    continue;
                };
                if event.state.is_pressed() {
                    keys_pressed.insert(key);
                } else {
                    keys_pressed.remove(&key);
                }
            }
            _ => {}
        }
    }

    for key in keys_pressed.iter() {
        const MOVE_SPEED: f64 = 3.0;
        let ms = (MOVE_SPEED * dt.0) as f32;
        match key {
            KeyCode::KeyW => {
                camera.translate((0.0, 0.0, ms));
            }
            KeyCode::KeyA => {
                camera.translate((-ms, 0.0, 0.0));
            }
            KeyCode::KeyS => {
                camera.translate((0.0, 0.0, -ms));
            }
            KeyCode::KeyD => {
                camera.translate((ms, 0.0, 0.0));
            }
            KeyCode::Space => {
                camera.translate((0.0, ms, 0.0));
            }
            KeyCode::ControlLeft => {
                camera.translate((0.0, -ms, 0.0));
            }
            _ => {}
        };
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct CameraData {
    pub position: [f32; 3],
    pub _pad0: u32,
    pub forward: [f32; 3],
    pub _pad1: u32,
    pub up: [f32; 3],
    pub _pad2: u32,
    pub dims: [f32; 2],
    pub focal_length: f32,
    pub changed: u32,
    pub _pad3: [u32; 1],
}

impl CameraData {
    pub fn new() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            forward: [0.0, 0.0, 1.0],
            up: [0.0, 1.0, 0.0],
            dims: [1.0, 1.0],
            focal_length: 1.0,
            ..Default::default()
        }
    }
}

#[derive(Component)]
pub struct Camera {
    pub data: CameraData,
    pub uniform: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub changed: bool,
}

impl Camera {
    pub fn new(device: &wgpu::Device, label: Option<&str>) -> Self {
        let label = label.unwrap_or_default();

        // let camera_data = CameraData {
        //     position: [-3.8, 0.4, 6.0],
        //     forward: [0.55, -0.59, 0.66],
        //     up: [0.31, 0.86, 0.38],
        //     dims: [1.0, 1.0],
        //     focal_length: 1.0,
        //     changed: 0,
        //     ..Default::default()
        // };

        let camera_data = CameraData::new();

        let uniform = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{} Camera Uniform", label)),
            contents: bytemuck::bytes_of(&camera_data),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(&format!("{} Camera Bindgroup Layout", label)),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{} Camera Bindgroup", label)),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform.as_entire_binding(),
            }],
        });

        Self {
            data: camera_data,
            uniform,
            bind_group,
            bind_group_layout,
            changed: false,
        }
    }

    pub fn update(&mut self, queue: &wgpu::Queue) {
        if self.changed {
            queue.write_buffer(&self.uniform, 0, bytemuck::bytes_of(&self.data));
            queue.submit([]);
            self.data.changed = 0;
        }
    }

    pub fn translate(&mut self, dir: impl Into<glam::Vec3>) {
        let dir = dir.into();
        let f = glam::Vec3::from(self.data.forward);
        let u = glam::Vec3::from(self.data.up);
        let r = u.cross(f).normalize();
        let mut pos = glam::Vec3::from_slice(&self.data.position);

        pos += dir.x * r;
        pos += dir.y * u;
        pos += dir.z * f;

        self.data.position = pos.to_array();
        self.data.changed = 1;
        self.changed = true;
    }

    pub fn rotate(&mut self, delta: impl Into<glam::Vec2>) {
        let delta = delta.into();
        let f = glam::Vec3::from(self.data.forward).normalize();
        let u = glam::Vec3::from(self.data.up).normalize();

        // Rotating about y axis for left/right is easy:
        let m = glam::Mat3::from_rotation_y(delta.x);
        let f = m * f;
        let u = m * u;

        // To rotate about r for up/down, we must rebase
        // so that r is x axis.
        let r = u.cross(f);
        let m = glam::Mat3::from_axis_angle(r, delta.y);
        let f = m * f;
        let u = m * u;

        self.data.forward = f.into();
        self.data.up = u.into();

        self.data.changed = 1;
        self.changed = true;
    }
}
