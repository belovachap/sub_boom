use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::{EventPump, TimerSubsystem};

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
const BUBBLE_MAX_FRAMES: u32 = 2 * FPS;

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


struct Bomb {
    bomb: Rect,
    frames: u32,
    max_frames: u32,
    hit_by_explosion: bool,
}

impl Bomb {
    fn new(rect: Rect, mf: u32) -> Self {
        Bomb {
            bomb: rect,
            frames: 0,
            max_frames: mf,
            hit_by_explosion: false,
        }
    }

    fn draw(&self, canvas: &mut Canvas<Window>) {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.fill_rect(self.bomb).unwrap();
    }
}

struct Bubble {
    bubble: Rect,
    frames: u32,
}

impl Bubble {
    fn new(rect: Rect, mf: u32) -> Self {
        Bubble {
            bubble: rect,
            frames: 0,
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
    missile_countdown: u32,
    hit_by_explosion: bool,
}

impl Submarine {
    fn new() -> Self {
        let mut rng = thread_rng();
        let x = rng.gen_range(0..=(DISPLAY_WIDTH - 50) as i32);
        let y = rng.gen_range(MIN_SUB_DEPTH..=(DISPLAY_HEIGHT - 20) as i32);
        Submarine {
            ship: Rect::new(x, y, 50, 20),
            sailing: SubDirection::Left,
            missile_countdown: 10 * FPS,
            hit_by_explosion: false,
        }
    }

    fn draw(&self, canvas: &mut Canvas<Window>) {
        canvas.set_draw_color(Color::RGB(255, 0, 0));
        canvas.fill_rect(self.ship).unwrap();
    }

    fn fire(&self, missiles: &mut Vec<Missile>) {
        let m = Missile::new(Rect::new(self.ship.x(), self.ship.y(), 6, 12));
        missiles.push(m);
    }
}

struct Missile {
    missile: Rect,
    hit_by_explosion: bool,
}

impl Missile {
    fn new(rect: Rect) -> Self {
        Missile {
            missile: rect,
            hit_by_explosion: false,
        }
    }

    fn draw(&self, canvas: &mut Canvas<Window>) {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.fill_rect(self.missile).unwrap();
    }
}

struct Explosion {
    blast: Rect,
    frames: u32,
    max_frames: u32,
}

impl Explosion {
    fn new(rect: Rect, mf: u32) -> Self {
        Explosion {
            blast: rect,
            frames: 0,
            max_frames: mf,
        }
    }

    fn draw(&self, canvas: &mut Canvas<Window>, bubbles: &mut Vec<Bubble>) {
        canvas.set_draw_color(Color::RGB(255, 102, 0));
        canvas.fill_rect(self.blast).unwrap();

        // Generate bubbles
        for _ in 0..25 {
            let mut rng = thread_rng();
            let x = rng.gen_range(self.blast.x()..self.blast.right());
            let y = rng.gen_range(self.blast.y()..self.blast.bottom());
            let bubble = Bubble::new(Rect::new(x, y, 1, 1), FPS);
            bubbles.push(bubble);
        }
    }
}

#[derive(Debug)]
enum GameState {
    Play,
    Quit,
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

    let mut game_state = GameState::Play;

    'running: loop {
        match game_state {
            GameState::Play => play_game(&timer, &mut event_pump, &mut canvas, &mut game_state),
            GameState::Quit => break 'running,
        }
    }
}

fn play_game(
    timer: &TimerSubsystem,
    event_pump: &mut EventPump,
    canvas: &mut Canvas<Window>,
    game_state: &mut GameState,
) {
    println!("MS_PER_FRAME: {}", MS_PER_FRAME);

    let mut destroyer = Destroyer::new();

    let mut bombs = Vec::<Bomb>::new();

    let mut add_sub_counter = ADD_SUB_FREQUENCY;
    let mut submarines = Vec::<Submarine>::new();

    let mut missiles = Vec::<Missile>::new();

    let mut explosions = Vec::<Explosion>::new();

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
                } => {
                    *game_state = GameState::Quit;
                    break 'running;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => {
                    // move ship left
                    let new_right = destroyer.ship.right() - 2;
                    let new_right = cmp::max(100, new_right);
                    destroyer.ship.set_right(new_right);
                    // Generate bubbles
                    for _ in 0..100 {
                        let mut rng = thread_rng();
                        let x =
                            rng.gen_range(destroyer.ship.right()..=(destroyer.ship.right() + 5));
                        let y = rng.gen_range((WATER_LEVEL + 1)..=(WATER_LEVEL + 10));
                        let bubble = Bubble::new(Rect::new(x, y, 1, 1), FPS);
                        bubbles.push(bubble);
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => {
                    let new_right = destroyer.ship.right() + 2;
                    let new_right = cmp::min(DISPLAY_WIDTH as i32, new_right);
                    destroyer.ship.set_right(new_right);
                    // Generate bubbles
                    for _ in 0..100 {
                        let mut rng = thread_rng();
                        let x = rng.gen_range((destroyer.ship.left() - 5)..=destroyer.ship.left());
                        let y = rng.gen_range((WATER_LEVEL + 1)..=(WATER_LEVEL + 10));
                        let bubble = Bubble::new(Rect::new(x, y, 1, 1), FPS);
                        bubbles.push(bubble);
                    }
                }
                  Event::KeyUp {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    let b = Bomb::new(Rect::new(destroyer.ship.x(), destroyer.ship.bottom(), 10, 10), 5 * FPS);
                    bombs.push(b);
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

        for e in explosions.iter() {
            if e.blast.has_intersection(destroyer.ship) {
                *game_state = GameState::Play;
                break 'running;
            }
        }

        destroyer.draw(canvas);

        for sub in submarines.iter_mut() {
            for e in explosions.iter() {
                if e.blast.has_intersection(sub.ship) {
                    sub.hit_by_explosion = true;
                    break;
                }
            }

            if sub.hit_by_explosion {
                let e = Explosion::new(sub.ship.clone(), 2 * FPS);
                explosions.push(e);
                continue;
            }

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

            // Should fire missile?
            sub.missile_countdown -= 1;
            if sub.missile_countdown <= 0 {
                sub.missile_countdown = 10 * FPS;
                sub.fire(&mut missiles);
            }

            sub.draw(canvas);
        }

        submarines.retain(|s| !s.hit_by_explosion);

        // Handle Missiles
        for m in missiles.iter_mut() {
            for e in explosions.iter() {
                if e.blast.has_intersection(m.missile) {
                    m.hit_by_explosion = true;
                    break;
                }
            }

            if m.hit_by_explosion {
                let e = Explosion::new(m.missile.clone(), 1 * FPS);
                explosions.push(e);
                continue;
            }

            // Generate bubbles
            let mut rng = thread_rng();
            let x: i32 = m.missile.center().x;
            let y: i32 = m.missile.bottom();
            let mf: u32 = rng.gen_range(10..FPS);
            let bubble = Bubble::new(Rect::new(x, y, 1, 1), mf);
            bubbles.push(bubble);

            let y = cmp::max(WATER_LEVEL, m.missile.y() - 2);
            m.missile.set_y(y);

            if y == WATER_LEVEL {
                let e = Explosion::new(m.missile.clone(), 1 * FPS);
                explosions.push(e);
            } else {
                m.draw(canvas);
            }
        }

        missiles.retain(|m| m.missile.y() > WATER_LEVEL);
        missiles.retain(|m| !m.hit_by_explosion);

        // Handle Bombs
        for b in bombs.iter_mut() {
            b.frames += 1;

            if b.frames > b.max_frames {
                let e = Explosion::new(b.bomb.clone(), 2 * FPS);
                explosions.push(e);
                continue;
            }

            for e in explosions.iter() {
                if e.blast.has_intersection(b.bomb) {
                    b.hit_by_explosion = true;
                    break;
                }
            }

            if b.hit_by_explosion {
                let e = Explosion::new(b.bomb.clone(), 2 * FPS);
                explosions.push(e);
                continue;
            }

            // Generate bubbles
            let mut rng = thread_rng();
            let x: i32 = b.bomb.center().x;
            let y: i32 = b.bomb.top();
            let mf: u32 = rng.gen_range(10..FPS);
            let bubble = Bubble::new(Rect::new(x, y, 1, 1), mf);
            bubbles.push(bubble);

            b.bomb.set_y(b.bomb.y() + 1);

            b.draw(canvas);
        }

        bombs.retain(|b| b.frames <= b.max_frames);
        bombs.retain(|b| !b.hit_by_explosion);

        // Remove old bubbles or surfacing bubbles
        bubbles.retain(|b| b.frames <= BUBBLE_MAX_FRAMES && b.bubble.y() > WATER_LEVEL);

        for b in bubbles.iter_mut() {
            b.frames += 1;
            let mut rng = thread_rng();
            let x: i32 = rng.gen_range(-1..2);
            let y: i32 = rng.gen_range(-1..0);
            b.bubble.set_y(b.bubble.y() + y);
            b.bubble.set_x(b.bubble.x() + x);
            b.draw(canvas);
        }

        // Handle Explosions
        explosions.retain(|e| e.frames <= e.max_frames);

        for e in explosions.iter_mut() {
            e.frames += 1;
            e.blast.set_x(e.blast.x() - 1);
            e.blast.set_y(e.blast.y() - 1);
            e.blast.set_width(e.blast.width() + 2);
            e.blast.set_height(e.blast.height() + 2);
            e.draw(canvas, &mut bubbles);
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
