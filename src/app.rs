use std::sync::Arc;

use crate::graphics::{create_graphics, Graphics};

use winit::{application::ApplicationHandler, dpi::{PhysicalPosition, PhysicalSize}, event::{DeviceEvent, WindowEvent}, event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy}, window::{Window, WindowId}};

enum State {
    Ready(Graphics),
    Init(Option<EventLoopProxy<Graphics>>),
}

pub struct App {
    state: State,
}

impl App {
    pub fn new(event_loop: &EventLoop<Graphics>) -> Self {
        Self {
            state: State::Init(Some(event_loop.create_proxy())),
        }
    }

    fn draw(&mut self) {
        if let State::Ready(gfx) = &mut self.state {
            gfx.draw();
        }
    }

    fn resized(&mut self, size: PhysicalSize<u32>) {
        if let State::Ready(gfx) = &mut self.state {
            gfx.resize(size);
        }
    }

    fn update(&mut self) {
        if let State::Ready(gfx) = &mut self.state {
            gfx.update();
        }
    }

    fn process_camera_event(&mut self, event: &DeviceEvent) {
        if let State::Ready(gfx) = &mut self.state {
            gfx.process_camera_event(event);
        }
    }

    fn update_cursor_position(&mut self, pos: PhysicalPosition<f64>) {
        if let State::Ready(gfx) = &mut self.state {
            gfx.update_cursor_position(pos);
        }
    }
}



impl ApplicationHandler<Graphics> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let State::Init(proxy) = &mut self.state {
            if let Some(proxy) = proxy.take() {
                let win_attr = Window::default_attributes().with_title("Sitelen");

                let window = Arc::new(
                    event_loop
                        .create_window(win_attr)
                        .expect("create window err."),
                );

                pollster::block_on(create_graphics(window, proxy));
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::Resized(size) => self.resized(size),
            WindowEvent::RedrawRequested => {
                self.update();
                self.draw();
            },
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::CursorMoved { device_id: _, position } => {
                self.update_cursor_position(position);
            }
            _ => {}
        }
    }

    fn device_event(
            &mut self,
            _event_loop: &ActiveEventLoop,
            _device_id: winit::event::DeviceId,
            event: winit::event::DeviceEvent,
        ) {
        self.process_camera_event(&event);
    }

    fn user_event(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, event: Graphics) {
        self.state = State::Ready(event);
    }
    

}