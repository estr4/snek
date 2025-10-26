#![allow(dead_code)]

const SCREEN_WIDTH: i32 = 1200;
const SCREEN_HEIGHT: i32 = 720;

use std::time::{Duration, Instant};

use rand::random;
use raylib::prelude::*;

#[derive(Copy, Clone, Debug)]
struct Anchor {
    point: Vector2,
    from_parent: Vector2,
    rhs: Vector2,
    distance: f32,
}

impl Anchor {
    fn new(point: Vector2, distance: f32) -> Self {
        Self {
            point,
            from_parent: Vector2 { x: 0.0, y: 0.0 },
            rhs: Vector2 { x: 0.0, y: 0.0 },
            distance,
        }
    }

    /// Update the Anchors position, such that the Anchor gets put the
    /// Parent's distance away from it.
    fn scale_to_dist(&mut self, parent: &Anchor) {
        self.from_parent = (self.point - parent.point)
            .normalized()
            .scale_by(parent.distance);

        self.point = parent.point + self.from_parent;

        self.rhs = Vector2::new(self.from_parent.y, -self.from_parent.x)
            .normalized()
            .scale_by(self.distance);
    }

    /// Draw the Anchor for debugging
    fn visualise(&self, d: &mut RaylibDrawHandle) {
        // Middle
        d.draw_circle_v(self.point, 3.0, Color::WHITE);

        // Perimiter
        d.draw_circle_lines_v(self.point, self.distance, Color::WHITE);

        // Connection
        d.draw_line_ex(self.point, self.point - self.from_parent, 3.0, Color::CYAN);

        // Edges
        d.draw_circle_v(self.point - self.rhs, 5.0, Color::RED);
        d.draw_circle_v(self.point + self.rhs, 5.0, Color::RED);
    }
}

#[derive(Debug, Clone)]
struct Snake {
    body: Vec<Anchor>,
}

impl Snake {
    fn new(size: usize) -> Self {
        let mut body = Vec::<Anchor>::with_capacity(size);
        for _ in 0..size {
            let anchor = Anchor::new(
                Vector2 {
                    x: random(),
                    y: random(),
                } * SCREEN_HEIGHT as f32,
                random::<f32>() * 50.0,
            );
            body.push(anchor);
        }
        Self { body }
    }

    /// Make A Snake from an existing Vec
    fn from_body(body: Vec<Anchor>) -> Self {
        Self { body }
    }


    /// Make the Snake respond to left and right input
    fn input(&mut self, rl: &mut RaylibHandle) {
        let a = self.body[1];
        let mut direction = -a.from_parent;

        if rl.is_key_down(KeyboardKey::KEY_LEFT) {
            direction -= a.rhs * 0.33;
        }

        if rl.is_key_down(KeyboardKey::KEY_RIGHT) {
            direction += a.rhs * 0.33;
        }

        self.body[0].point += direction.normalized() * 7.5;
    }

    /// Update the Snakes body based on the anchor positions
    fn update(&mut self) {
        for i in 0..self.body.len() - 1 {
            let a = self.body[i];
            let mut b = self.body[i + 1];
            b.scale_to_dist(&a);

            self.body[i + 1] = b;
        }

        self.body[0].from_parent = (self.body[1].point - self.body[0].point)
            .normalized()
            .scale_by(self.body[0].distance);

        self.body[0].rhs = Vector2::new(self.body[0].from_parent.y, -self.body[0].from_parent.x)
            .normalized()
            .scale_by(self.body[0].distance);
    }

    fn visualise(&self, d: &mut RaylibDrawHandle) {
        for a in self.body.iter() {
            a.visualise(d);
        }
    }

    fn draw(&self, d: &mut RaylibDrawHandle) {
        let mut points = vec![];
        for a in self.body.iter() {
            points.push(a.point + a.rhs);
            points.push(a.point - a.rhs);
        }   

        // Body
        d.draw_triangle_strip(&points, Color::DARKOLIVEGREEN);

        // Head And Tail
        d.draw_circle_v(self.body[0].point, self.body[0].distance, Color::DARKOLIVEGREEN);
        d.draw_circle_v(self.body[self.body.len() - 1].point, self.body[self.body.len() - 1].distance, Color::DARKOLIVEGREEN);

        // Eyes
        d.draw_circle_v(
            self.body[0].point + self.body[0].rhs + self.body[0].from_parent * 0.3,
            5.0, Color::BLACK
        );

        d.draw_circle_v(
            self.body[0].point - self.body[0].rhs + self.body[0].from_parent * 0.3,
            5.0, Color::BLACK
        );
    }
}

#[derive(Debug, Copy, Clone)]
struct Apple {
    point: Vector2,
    radius: f32,
}

impl Apple {
    fn new(point: Vector2, radius: f32) -> Self {
        Self { point, radius }
    }

    fn draw(&self, d: &mut RaylibDrawHandle) {
        d.draw_circle_v(self.point, self.radius, Color::ORANGERED);
    }
}


struct Game {
    score: i32,
    screen: Screen,
    debug: bool,
    apple: Apple,
    snake: Snake,
    body: Vec::<Anchor>,
    start: Instant
}

impl Game {
    pub fn new() -> Self {
        let mut body = Vec::<Anchor>::with_capacity(20);
        
        let sizes: [f32; 13] = [
            33.0, 40.0, 42.0, 33.0, 30.0, 30.0, 30.0, 30.0, 30.0, 30.0, 30.0, 30.0, 30.0
        ];

        for i in 0..12 {
            let sum: f32 = sizes[0..i].iter().sum();
            let anchor = Anchor::new(
                Vector2 {
                    x: 50.0 - sum,
                    y: 120.0,
                },
                sizes[i],
            );
            body.push(anchor);
        }

        let point = Vector2::new(
            random::<f32>() * SCREEN_WIDTH as f32,
            random::<f32>() * SCREEN_HEIGHT as f32,
        );
        
        let apple = Apple::new(point, 20.0);
        let snake = Snake::from_body(body.clone());
        Self {
            score: 0,
            screen: Screen::Logo,
            debug: false,
            apple,
            snake,
            body,
            start: Instant::now(),
        }
    }

    /// Try to Eat the Apple
    fn try_apple(&mut self) {
        let head = self.snake.body[0];
        if head.distance + self.apple.radius > (head.point - self.apple.point).length() - 5.0 {
            self.snake.body.push(
                Anchor::new(self.snake.body[self.snake.body.len() - 1].point, 30.0),
            );

            self.apple.point = Vector2::new(
                random::<f32>() * SCREEN_WIDTH as f32,
                random::<f32>() * SCREEN_HEIGHT as f32,
            );

            self.score += 1;
        }
    }
    
    /// Check If The Head collides with the Tail
    fn oroboros(&mut self) {
        let head = self.snake.body[0];
        for tail in self.snake.body[3..].iter() {
            if head.distance + tail.distance > (head.point - tail.point).length() {
                self.screen = Screen::GameOver;
            }
        }
    }

    fn clamp(&mut self) {
        let d = self.snake.body[0].distance;
        self.snake.body[0].point.x = self.snake.body[0].point.x.clamp(0.0 + d, SCREEN_WIDTH as f32 - d);
        self.snake.body[0].point.y = self.snake.body[0].point.y.clamp(0.0 + d, SCREEN_HEIGHT as f32 - d);
    }

    fn timer(&mut self) {
        let duration = Duration::from_secs(40);
        if Instant::now() - self.start >= duration {
            self.screen = Screen::GameOver;
        }
    }

    fn logo(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) {
        if rl.is_key_pressed(KeyboardKey::KEY_ENTER) {
            self.start = Instant::now();
            self.screen = Screen::Play;
        }
        
        let mut d = rl.begin_drawing(&thread);
        d.draw_text("WELCOME", 10, 10, 60, Color::WHITE);
        d.draw_text("YOU HAVE 40 SECONDS TO EAT AS MANY ORANGES AS YOU CAN.", 10, 140, 30, Color::WHITE);
        d.draw_text("PRESS ENTER TO START", 10, 200, 30, Color::GRAY);
        d.clear_background(Color::new(50, 50, 50, 255));
    }

    fn game_over(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) {
        if rl.is_key_pressed(KeyboardKey::KEY_ENTER) {
            self.score = 0;
            self.screen = Screen::Play;
            self.snake = Snake::from_body(self.body.clone());
            self.start = Instant::now();
            self.apple.point = Vector2::new(
                random::<f32>() * SCREEN_WIDTH as f32,
                random::<f32>() * SCREEN_HEIGHT as f32,
            );
        }
        
        let mut d = rl.begin_drawing(&thread);
        d.draw_text("GAME OVER", 10, 10, 60, Color::RED);
        d.draw_text(&format!("FINAL SCORE: {}", self.score), 10, 140, 30, Color::WHITE);
        d.draw_text("PRESS ENTER TO RESTART", 10, 200, 30, Color::GRAY);
        d.clear_background(Color::new(50, 50, 50, 255));
    }

    fn play(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) {
        
        if rl.is_key_pressed(KeyboardKey::KEY_COMMA) {
            self.debug = !self.debug;
        }
        
        self.snake.input(rl);
        self.try_apple();
        self.snake.update();

        let mut d = rl.begin_drawing(&thread);
        self.clamp();
        self.oroboros();
        self.timer();
        
        self.apple.draw(&mut d);
        self.snake.draw(&mut d);


        if self.debug {
            self.snake.visualise(&mut d);            
        }
        
        d.draw_text(&format!("Score: {}", self.score), 10, 10, 20, Color::WHITE);
        d.draw_text(&format!("Time: {:.1}s/40s", (Instant::now() - self.start).as_secs_f32()), SCREEN_WIDTH - 160, 10, 20, Color::WHITE);
        d.clear_background(Color::DARKSLATEGRAY);
    }

    fn run(&mut self) {
        let (mut rl, thread) = raylib::init()
            .size(SCREEN_WIDTH, SCREEN_HEIGHT)
            .title("SNEK")
            .build();

        rl.set_target_fps(60);
        
        while !rl.window_should_close() {
            match self.screen {
                Screen::Logo => self.logo(&mut rl, &thread),
                Screen::Play => self.play(&mut rl, &thread),
                Screen::GameOver => self.game_over(&mut rl, &thread),
            };
        }
    }
}

enum Screen {
    Logo,
    Play,
    GameOver,
}

fn main() {
    let mut game = Game::new();
    game.run();
}
