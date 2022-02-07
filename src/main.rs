use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use rand::prelude::*;
use std::cmp;
use std::time::Duration;

const DISPLAY_WIDTH: u32 = 800;
const DISPLAY_HEIGHT: u32 = 600;
const WATER_LEVEL: i32 = 70; // Constant that determines where the ocean is drawn
const MIN_SUB_DEPTH: i32 = WATER_LEVEL + 20; // Subs should be at this depth or lower
const FPS: u32 = 30;
const MS_PER_FRAME: u32 = 1000 / FPS;
const ADD_SUB_FREQUENCY: u32 = 15 * FPS;

struct Destroyer {
    ship: Rect,
}

impl Destroyer {
    fn new() -> Self {
        Destroyer {
            ship: Rect::new(50, 50, 100, 20),
        }
    }

    fn draw(&self, canvas: &mut Canvas<Window>) {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.fill_rect(self.ship).unwrap();
    }
}

struct Bubble {
    bubble: Rect,
    frames: u32,
    max_frames: u32,
}

impl Bubble {
    fn new(rect: Rect, mf: u32) -> Self {
        Bubble {
            bubble: rect,
            frames: 0,
            max_frames: mf,
        }
    }

    fn draw(&self, canvas: &mut Canvas<Window>) {
        canvas.set_draw_color(Color::RGB(128, 128, 255));
        canvas.fill_rect(self.bubble).unwrap();
    }
}

#[derive(Debug)]
enum SubDirection {
    Left,
    Right,
}

struct Submarine {
    ship: Rect,
    sailing: SubDirection,
}

impl Submarine {
    fn new() -> Self {
        let mut rng = thread_rng();
        let depth: i32 = rng.gen_range(MIN_SUB_DEPTH..(DISPLAY_HEIGHT - 20) as i32);
        Submarine {
            ship: Rect::new(50, depth, 50, 20),
            sailing: SubDirection::Left,
        }
    }

    fn draw(&self, canvas: &mut Canvas<Window>) {
        canvas.set_draw_color(Color::RGB(255, 0, 0));
        canvas.fill_rect(self.ship).unwrap();
    }
}

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

    let mut destroyer = Destroyer::new();

    let mut add_sub_counter = ADD_SUB_FREQUENCY;
    let mut submarines = Vec::<Submarine>::new();

    let mut bubbles = Vec::<Bubble>::new();

    'running: loop {
        let frame_start_time = timer.ticks();

        add_sub_counter += 1;
        if add_sub_counter >= ADD_SUB_FREQUENCY {
            add_sub_counter = 0;
            submarines.push(Submarine::new());
        }

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => {
                    // move ship left
                    let new_right = destroyer.ship.right() - 5;
                    let new_right = cmp::max(100, new_right);
                    destroyer.ship.set_right(new_right);
                    // Generate bubbles
                    let bubble = Bubble::new(
                        Rect::new(destroyer.ship.right() + 1, WATER_LEVEL + 5, 1, 1),
                        FPS,
                    );
                    bubbles.push(bubble);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => {
                    let new_right = destroyer.ship.right() + 5;
                    let new_right = cmp::min(DISPLAY_WIDTH as i32, new_right);
                    destroyer.ship.set_right(new_right);
                    // Generate bubbles
                    let bubble = Bubble::new(
                        Rect::new(destroyer.ship.left() - 1, WATER_LEVEL + 5, 1, 1),
                        FPS,
                    );
                    bubbles.push(bubble);
                }
                _ => {}
            }
        }

        canvas.set_draw_color(Color::RGB(255, 255, 255));
        canvas.clear();

        canvas.set_draw_color(Color::RGB(0, 0, 255));
        canvas
            .fill_rect(Rect::new(0, WATER_LEVEL, DISPLAY_WIDTH, DISPLAY_HEIGHT))
            .unwrap();

        destroyer.draw(&mut canvas);

        for sub in submarines.iter_mut() {
            let mut new_right;
            match sub.sailing {
                SubDirection::Left => {
                    new_right = sub.ship.right() - 1;
                    // Generate bubbles
                    let mut rng = thread_rng();
                    let x: i32 = rng.gen_range(sub.ship.right()..(sub.ship.right() + 5));
                    let y: i32 =
                        rng.gen_range((sub.ship.center().y - 3)..(sub.ship.center().y + 3));
                    let mf: u32 = rng.gen_range(10..FPS);
                    let bubble = Bubble::new(Rect::new(x, y, 1, 1), mf);
                    bubbles.push(bubble);
                }
                SubDirection::Right => {
                    new_right = sub.ship.right() + 1;
                    // Generate bubbles
                    let mut rng = thread_rng();
                    let x: i32 = rng.gen_range((sub.ship.left() - 5)..sub.ship.left());
                    let y: i32 =
                        rng.gen_range((sub.ship.center().y - 3)..(sub.ship.center().y + 3));
                    let mf: u32 = rng.gen_range(10..FPS);
                    let bubble = Bubble::new(Rect::new(x, y, 1, 1), mf);
                    bubbles.push(bubble);
                }
            }

            if new_right >= DISPLAY_WIDTH as i32 {
                new_right = DISPLAY_WIDTH as i32;
                sub.sailing = SubDirection::Left;
            } else if new_right <= 50 {
                new_right = 50;
                sub.sailing = SubDirection::Right;
            }
            sub.ship.set_right(new_right);
            sub.draw(&mut canvas);
        }

        // Remove old bubbles or surfacing bubbles
        bubbles.retain(|b| b.frames <= b.max_frames && b.bubble.y() > WATER_LEVEL);

        for b in bubbles.iter_mut() {
            b.frames += 1;
            let mut rng = thread_rng();
            let x: i32 = rng.gen_range(-1..2);
            let y: i32 = rng.gen_range(-1..0);
            b.bubble.set_y(b.bubble.y() + y);
            b.bubble.set_x(b.bubble.x() + x);
            b.draw(&mut canvas);
        }

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
