use enigo::{Button, Coordinate, Direction, Enigo, Mouse, Settings};
use winit::dpi::PhysicalSize;


pub fn move_mouse(x_pixels: i32, y_pixels: i32, coordinate: Coordinate) {
    let mut enigo = Enigo::new(&Settings::default()).unwrap();
    enigo.move_mouse(x_pixels, y_pixels, coordinate).unwrap();
}

pub fn jump_mouse_to(monitor_size: PhysicalSize<u32>, x_pos: fn(i32, i32) -> i32, y_pos: fn(i32, i32) -> i32) {
    let enigo = Enigo::new(&Settings::default()).unwrap();
    let mouse_location = enigo.location().unwrap();

    let mouse_x = mouse_location.0;
    let mouse_y = mouse_location.1;
    let width = monitor_size.width as i32;
    let height = monitor_size.height as i32;

    let x_pixels = x_pos(mouse_x, width);
    let y_pixels = y_pos(mouse_y, height);

    move_mouse(x_pixels, y_pixels, Coordinate::Abs);
}

pub fn click_mouse_button(button: Button) {
    let mut enigo = Enigo::new(&Settings::default()).unwrap();
    enigo.button(button, Direction::Press).unwrap();
    enigo.button(button, Direction::Release).unwrap();
}