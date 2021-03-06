use std::{env, fs::File, io::Read, thread::sleep, time::Duration};

use hardware::{Keyboard, CPU, DISPLAY_HEIGHT, DISPLAY_WIDTH};
use pixels::{Pixels, SurfaceTexture};
use rodio::Sink;
use winit::{
    dpi::LogicalSize,
    event::{Event, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

mod hardware;

const WINDOW_WIDTH: u32 = 640;
const WINDOW_HEIGHT: u32 = 320;

fn read_bytes_from_file(rom_name: &str) -> Vec<u8> {
    let mut f =
        File::open(rom_name).unwrap_or_else(|_| panic!("Unable to open file: {}", rom_name));
    let mut rom = Vec::new();
    f.read_to_end(&mut rom).expect("Unable to read rom");

    rom
}

fn get_rom_data() -> Vec<u8> {
    let rom_name = env::args().nth(1).expect("No file name given for ROM");
    read_bytes_from_file(&rom_name)
}

fn setup_audio() -> Sink {
    let device = rodio::default_output_device().unwrap();
    let audio_sink = Sink::new(&device);
    let audio_source = rodio::source::SineWave::new(440);
    audio_sink.append(audio_source);
    audio_sink.pause();

    audio_sink
}

fn setup_hardware() -> (CPU, Keyboard) {
    let beeper = setup_audio();

    let mut cpu = CPU::new(beeper);
    let keyboard = Keyboard::new();

    let rom_data = get_rom_data();

    cpu.load_rom(&rom_data);

    (cpu, keyboard)
}

pub fn run() {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();

    let window = {
        let size = LogicalSize::new(WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Chip-8 Emulator")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(DISPLAY_WIDTH as u32, DISPLAY_HEIGHT as u32, surface_texture).unwrap()
    };

    let (mut cpu, mut keyboard) = setup_hardware();

    event_loop.run(move |event, _, control_flow| {
        if let Event::RedrawRequested(_) = event {
            if pixels.render().is_err() {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        if input.update(&event) {
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            keyboard.handle_input(&input);

            if let Some(size) = input.window_resized() {
                pixels.resize(size.width, size.height)
            }
        }

        cpu.step(pixels.get_frame(), &keyboard);

        window.request_redraw();

        // Sleep at a rate that emulates about 500Hz. This won't be accurate.
        sleep(Duration::new(0, 2_000_000))
    });
}
