#![windows_subsystem = "windows"]

mod mouse_handler;
use std::collections::HashSet;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

use enigo::{Button, Coordinate};
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalSize, Size};
use winit::event::ElementState::Pressed;
use winit::event::ElementState::Released;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

struct App {
    window: Option<Window>,
    jump_mode_enabled: bool,
    sender: Sender<ChannelData>,
}

#[derive(Debug, PartialEq, Hash)]
pub enum WindowMovement {
    Top,
    Down,
    Left,
    Right,
}

impl Eq for WindowMovement {}

#[derive(Debug)]
enum ChannelData {
    Movement(WindowMovement, ElementState),
}

fn handle_movement(
    movements: &mut HashSet<WindowMovement>,
    movement: WindowMovement,
    state: ElementState,
) {
    match state {
        Pressed => {
            movements.insert(movement);
        }
        Released => {
            movements.remove(&movement);
        }
    }
    let step = 5;
    let (x_pixels, y_pixels) = movements.iter().fold((0, 0), |acc, next| match next {
        WindowMovement::Top => (acc.0, acc.1 - step),
        WindowMovement::Down => (acc.0, acc.1 + step),
        WindowMovement::Left => (acc.0 - step, acc.1),
        WindowMovement::Right => (acc.0 + step, acc.1),
    });

    mouse_handler::move_mouse(x_pixels, y_pixels, Coordinate::Rel);
}

fn run_input_handler_thread(receiver: Receiver<ChannelData>) {
    let mut movements: HashSet<WindowMovement> = HashSet::new();
    loop {
        let received = receiver.try_recv();

        match received {
            Ok(channel_data) => match channel_data {
                ChannelData::Movement(movement, state) => {
                    handle_movement(&mut movements, movement, state)
                }
            },
            _ => {}
        }
        thread::sleep(Duration::from_millis(5));
    }
}

fn jump(movement: WindowMovement, monitor_size: PhysicalSize<u32>) {
    match movement {
        WindowMovement::Top => {
            mouse_handler::jump_mouse_to(monitor_size, |mouse_x, _| mouse_x, |mouse_y, _| mouse_y / 2)
        }
        WindowMovement::Down => {
            mouse_handler::jump_mouse_to(monitor_size, |mouse_x, _| mouse_x, |mouse_y, height| (height + mouse_y) / 2)
        }
        WindowMovement::Left => {
            mouse_handler::jump_mouse_to(monitor_size, |mouse_x, _| mouse_x / 2, |mouse_y, _| mouse_y)
        }
        WindowMovement::Right => {
            mouse_handler::jump_mouse_to(monitor_size, |mouse_x, width| (mouse_x + width) / 2, |mouse_y, _| mouse_y)
        }
    }
}

fn map_to_movement(physical_key: PhysicalKey) -> Option<WindowMovement> {
    match physical_key {
        PhysicalKey::Code(KeyCode::KeyW) => Some(WindowMovement::Top),
        PhysicalKey::Code(KeyCode::KeyS) => Some(WindowMovement::Down),
        PhysicalKey::Code(KeyCode::KeyA) => Some(WindowMovement::Left),
        PhysicalKey::Code(KeyCode::KeyD) => Some(WindowMovement::Right),
        _ => None,
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title("Keyboard Mouse")
            .with_inner_size(Size::Physical(PhysicalSize::new(0, 0)))
            .with_transparent(true)
            .with_decorations(false);

        self.window = Some(event_loop.create_window(window_attributes).unwrap());
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Focused(focus) => {
                if !focus {
                    let unwrapped_window = self.window.as_ref().unwrap();
                    unwrapped_window.focus_window();
                }
            }
            WindowEvent::RedrawRequested => {
                let unwrapped_window = self.window.as_ref().unwrap();
                unwrapped_window.request_redraw();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::KeyQ),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                event_loop.exit();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::KeyJ),
                        state: ElementState::Pressed,
                        repeat: false,
                        ..
                    },
                ..
            } => {
                self.jump_mode_enabled = !self.jump_mode_enabled;
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key,
                        state,
                        repeat,
                        ..
                    },
                ..
            } => {
                if let Some(movement) = map_to_movement(physical_key) {
                    if self.jump_mode_enabled {
                        if state == Pressed && !repeat {
                            let window = self.window.as_ref().unwrap();
                            let monitor_size =
                                window.current_monitor().map(|monitor| monitor.size());

                            match monitor_size {
                                Some(size) => jump(movement, size),
                                None => {}
                            }
                        }
                    } else {
                        let _ = self.sender.send(ChannelData::Movement(movement, state));
                    }
                } else if state == ElementState::Pressed {
                    match physical_key {
                        PhysicalKey::Code(KeyCode::KeyC) => mouse_handler::click_mouse_button(Button::Left),
                        PhysicalKey::Code(KeyCode::KeyV) => mouse_handler::click_mouse_button(Button::Right),
                        _ => {}
                    }
                }
            }
            _ => (),
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);

    let (tx, rx): (Sender<ChannelData>, Receiver<ChannelData>) = mpsc::channel();

    thread::spawn(move || run_input_handler_thread(rx));

    let mut app = App {
        window: None,
        jump_mode_enabled: false,
        sender: tx,
    };

    event_loop
        .run_app(&mut app)
        .expect("Failed to run Event Loop");
}
