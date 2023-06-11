use cgmath::Vector2;
use error_iter::ErrorIter as _;
use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::{Event, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;
use std::time::Instant;

mod raycaster;

const WIDTH: usize = 600;
const HEIGHT: usize = 500;

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();

    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        let scaled_size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Raycaster")
            .with_inner_size(scaled_size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut last_frame_time = Instant::now();
    let mut delta_time = 0.0;

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH as u32, HEIGHT as u32, surface_texture)?
    };

    let mut camera = raycaster::Camera::new(Vector2::new(5.0, 5.0), 0.0, 60f64.to_radians());
    let size = 10;
    let mut map = raycaster::Map::new(size, size);

    //let texture = map.new_texture("textures/wall1.png");
    //let wall = map.new_tile(raycaster::Tile{
    //    color: raycaster::WallColor::TEXTURE(texture),
    //});
    let wall = raycaster::Tile::new(
        raycaster::Shape::Box,
        vec![
            raycaster::Color::Test2,
            raycaster::Color::Test,
            raycaster::Color::Test,
            raycaster::Color::Test,
        ],
        raycaster::Color::Test, 0.0,
        raycaster::Color::Test, 0.0,
    );
    for i in 0..size {
        map.set_tile(i, 0, wall);
        map.set_tile(i, size - 1, wall);
        map.set_tile(0, i, wall);
        map.set_tile(size - 1, i, wall);
    }

    let wall2 = raycaster::Tile::new(
        raycaster::Shape::Circle(raycaster::Circle {
            pos: Vector2 { x: 0.5, y: 0.5 },
            radius: 0.5,
        }),
        vec![raycaster::Color::Test2],
        raycaster::Color::Test, 0.0,
        raycaster::Color::Test, 0.0,
    );
    map.set_tile(5, 5, wall2);

    let wall3 = raycaster::Tile::new(
        raycaster::Shape::AxisAlignedBox(raycaster::AxisAlignedBox {
            min: Vector2 { x: 0.2, y: 0.2 },
            max: Vector2 { x: 0.3, y: 0.8 },
        }),
        vec![
            raycaster::Color::Test,
            raycaster::Color::Solid([1.0, 1.0, 1.0, 0.0]),
            raycaster::Color::Test,
            raycaster::Color::Test,
        ],
        raycaster::Color::Test, 0.0,
        raycaster::Color::Test, 0.0,
    );
    map.set_tile(6, 5, wall3);

    let wall4 = raycaster::Tile::new(
        raycaster::Shape::Line(raycaster::Line::new(
            Vector2 { x: 0.0, y: 0.0 },
            Vector2 { x: 1.0, y: 1.0 },
        )),
        vec![raycaster::Color::Test2, raycaster::Color::Test],
        raycaster::Color::Test, 0.0,
        raycaster::Color::Test, 0.0
    );
    map.set_tile(7, 5, wall4); 

    let wall5 = raycaster::Tile::new(
        raycaster::Shape::Void,
        vec![],
        raycaster::Color::Test, 0.3,
        raycaster::Color::Test, -0.2,
    );
    map.set_tile(4, 4, wall5); 



    let mut renderer = raycaster::Renderer::new(WIDTH, HEIGHT);

    event_loop.run(move |event, _, control_flow| {
        if let Event::RedrawRequested(_) = event {
            delta_time = last_frame_time.elapsed().as_secs_f64();
            println!("Delta time: {}ms", delta_time*1000.0);
            last_frame_time = Instant::now();
            renderer.render(pixels.frame_mut(), &camera, &map);
            if let Err(err) = pixels.render() {
                log_error("pixels.render", err);
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        if input.update(&event) {
            if input.key_pressed(VirtualKeyCode::Escape)
                || input.close_requested()
                || input.destroyed()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }

            const MOVE_SPEED: f64 = 3.0;
            if input.key_held(VirtualKeyCode::W) {
                camera.translate(Vector2::new(0.0, MOVE_SPEED*delta_time));
            }
            if input.key_held(VirtualKeyCode::S) {
                camera.translate(Vector2::new(0.0, -MOVE_SPEED*delta_time));
            }

            const ROT_SPEED: f64 = 2.0;
            if input.key_held(VirtualKeyCode::D) {
                camera.rotate(ROT_SPEED*delta_time);
            }
            if input.key_held(VirtualKeyCode::A) {
                camera.rotate(-ROT_SPEED*delta_time);
            }

            const Z_SPEED: f64 = 1.0;
            if input.key_held(VirtualKeyCode::Up) {
                camera.translate_z(Z_SPEED*delta_time);
            }
            if input.key_held(VirtualKeyCode::Down) {
                camera.translate_z(-Z_SPEED*delta_time);
            }

            if let Some(size) = input.window_resized() {
                if let Err(err) = pixels.resize_surface(size.width, size.height) {
                    log_error("pixels.resize_surface", err);
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }
            window.request_redraw();
        }
    });
}

fn log_error<E: std::error::Error + 'static>(method_name: &str, err: E) {
    error!("{method_name}() failed: {err}");
    for source in err.sources().skip(1) {
        error!("  Caused by: {source}");
    }
}
