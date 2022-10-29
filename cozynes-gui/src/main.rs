use std::path::PathBuf;

use clap::Parser;
use macroquad::{prelude::*, ui::root_ui};

use cozynes::{bus::Bus, cpu::Cpu, mem::Mem, rom::Rom};

fn window_conf() -> Conf {
    Conf {
        window_title: "NES emulator".into(),
        window_width: 320 * 4,
        window_height: 320 * 4,
        window_resizable: false,
        high_dpi: true,
        ..Default::default()
    }
}

const SCREEN_WIDTH: usize = 32;
const SCREEN_HEIGHT: usize = 32;

fn color(byte: u8) -> [u8; 4] {
    match byte {
        0 => BLACK,
        1 => WHITE,
        2 | 9 => GRAY,
        3 | 10 => RED,
        4 | 11 => GREEN,
        5 | 12 => BLUE,
        6 | 13 => MAGENTA,
        7 | 14 => YELLOW,
        _ => LIME,
    }
    .into()
}

fn read_screen_state(cpu: &Cpu, frame: &mut [u8]) -> bool {
    let mut update = false;
    for (chunk, addr) in frame.chunks_exact_mut(4).zip(0x0200..0x0600) {
        let color = color(cpu.read_byte(addr));
        if *chunk != color {
            update = true;
            chunk.copy_from_slice(&color);
        }
    }
    update
}

fn handle_input(cpu: &mut Cpu) {
    if is_key_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }
    if is_key_pressed(KeyCode::Up) {
        cpu.write_byte(0xff, 0x77);
    }
    if is_key_pressed(KeyCode::Down) {
        cpu.write_byte(0xff, 0x73);
    }
    if is_key_pressed(KeyCode::Left) {
        cpu.write_byte(0xff, 0x61);
    }
    if is_key_pressed(KeyCode::Right) {
        cpu.write_byte(0xff, 0x64);
    }
    if is_key_pressed(KeyCode::R) {
        cpu.memory = [0; 0xFFFF];
        cpu.reset();
        cpu.running = true;
    }
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    rom: PathBuf,
}

#[macroquad::main(window_conf)]
async fn main() {
    pretty_env_logger::init();

    let cli = Cli::parse();

    let file = std::fs::read(cli.rom).unwrap();
    let rom = Rom::new(&file).unwrap();
    let bus = Bus::new(rom);
    let mut cpu = Cpu::new(bus);
    cpu.running = true;

    rand::srand(std::time::Instant::now().elapsed().as_millis() as u64);

    let w = screen_width() as usize;
    let h = screen_height() as usize;

    let mut image = Image::gen_image_color(SCREEN_WIDTH as u16, SCREEN_HEIGHT as u16, BLUE);

    let texture = Texture2D::from_image(&image);
    texture.set_filter(FilterMode::Nearest);

    loop {
        clear_background(WHITE);
        read_screen_state(&cpu, &mut image.bytes);
        texture.update(&image);
        root_ui().label(None, &format!("FPS: {}", get_fps()));
        draw_texture_ex(
            texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(w as f32, h as f32)),
                ..Default::default()
            },
        );
        for _ in 0..200 {
            let random = rand::gen_range(1, 16);
            handle_input(&mut cpu);
            if cpu.running {
                cpu.write_byte(0xFE, random);
                cpu.step();
            }
        }
        next_frame().await;
    }
}
