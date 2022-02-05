use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

use std::time::Duration;

const DISPLAY_WIDTH: u32 = 640;
const DISPLAY_HEIGHT: u32 = 480;
const WATER_LEVEL: i32 = 70; // Constant that determines where the ocean is drawn
const MIN_SUB_DEPTH: i32 = WATER_LEVEL + 20; // Subs should be at this depth or lower
const FPS: u32 = 30;
const MS_PER_FRAME: u32 = 1000 / FPS;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let timer = sdl_context.timer().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Sub Boom!", DISPLAY_WIDTH, DISPLAY_HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut canvas = window.into_canvas().build().unwrap();

    println!("MS_PER_FRAME: {}", MS_PER_FRAME);
    'running: loop {
        let frame_start_time = timer.ticks();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        canvas.set_draw_color(Color::RGB(255, 255, 255));
        canvas.clear();

        canvas.set_draw_color(Color::RGB(0, 0, 255));
        canvas
            .fill_rect(Rect::new(0, WATER_LEVEL, DISPLAY_WIDTH, DISPLAY_HEIGHT))
            .unwrap();

        canvas.present();

        let frame_time = timer.ticks() - frame_start_time;
        println!("Frame rendered in {} milliseconds.", frame_time);

        if frame_time < MS_PER_FRAME {
            println!(
                "Sleeping {} milliseconds until next frame loop.",
                MS_PER_FRAME - frame_time
            );
            std::thread::sleep(Duration::from_millis((MS_PER_FRAME - frame_time).into()));
        }
    }
}
