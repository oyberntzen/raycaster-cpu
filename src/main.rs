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

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH as u32, HEIGHT as u32, surface_texture)?
    };

    let mut camera = raycaster::Camera::new(Vector2::new(5.0, 5.0), 0.0, 60f64.to_radians());
    let mut map = raycaster::Map::new(10, 10);

    let texture = map.new_texture("textures/wall1.png");
    let wall = map.new_tile(raycaster::Tile{
        color: raycaster::WallColor::TEXTURE(texture),
    });
    for i in 0..10 {
        map.set_tile(i, 0, wall);
        map.set_tile(i, 9, wall);
        map.set_tile(0, i, wall);
        map.set_tile(9, i, wall);
    }

    event_loop.run(move |event, _, control_flow| {
        if let Event::RedrawRequested(_) = event {
            raycaster::render(pixels.frame_mut(), WIDTH, HEIGHT, &camera, &map);
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

            const MOVE_SPEED: f64 = 3.0 / 60.0;
            if input.key_held(VirtualKeyCode::W) {
                camera.translate(Vector2::new(0.0, MOVE_SPEED));
            }
            if input.key_held(VirtualKeyCode::S) {
                camera.translate(Vector2::new(0.0, -MOVE_SPEED));
            }

            const ROT_SPEED: f64 = 2.0 / 60.0;
            if input.key_held(VirtualKeyCode::D) {
                camera.rotate(ROT_SPEED);
            }
            if input.key_held(VirtualKeyCode::A) {
                camera.rotate(-ROT_SPEED);
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
