use winit::{
    dpi::PhysicalPosition,
    event::{DeviceEvent, ElementState, MouseScrollDelta},
    window::Window,
};

use super::OrbitCamera;


#[derive(Debug)]
pub struct CameraController {
    pub rotate_speed: f32,
    pub zoom_delta: f32,
    is_drag_rotate: bool,
    is_left_down: bool,
    is_right_down: bool,
}

impl CameraController {
    pub fn new(rotate_speed: f32, zoom_delta: f32) -> Self {
        Self {
            rotate_speed,
            zoom_delta,
            is_drag_rotate: false,
            is_left_down: false,
            is_right_down: false,
        }
    }

    pub fn process_events(
        &mut self,
        event: &DeviceEvent,
        window: &Window,
        camera: &mut OrbitCamera,
    ) {
        match event {
            DeviceEvent::Button { button: 2, state } => {
                self.is_drag_rotate = *state == ElementState::Pressed;
            },
            DeviceEvent::Button { button: 1, state } => {
                self.is_left_down = *state == ElementState::Pressed;
            },
            DeviceEvent::Button { button: 3, state } => {
                self.is_right_down = *state == ElementState::Pressed;
            },
            DeviceEvent::MouseWheel { delta, .. } => {
                let scroll_amount = match delta {
                    // A mouse line is about 1 px.
                    MouseScrollDelta::LineDelta(_, scroll) => *scroll as f32,
                    MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => {
                        *scroll as f32
                    }
                };

                if scroll_amount.is_sign_positive() {
                    camera.add_distance(1.0 * self.zoom_delta);
                } else {
                    camera.add_distance(-1.0 * self.zoom_delta);
                }

                window.request_redraw();
            }
            DeviceEvent::MouseMotion { delta } => {
                if self.is_drag_rotate {
                    camera.add_yaw(-delta.0 as f32 * self.rotate_speed);
                    camera.add_pitch(delta.1 as f32 * self.rotate_speed);
                    window.request_redraw();
                }

                if self.is_left_down {
                }
            },
            _ => (),
        }
    }
}
