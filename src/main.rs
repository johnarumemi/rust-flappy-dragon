use std::fmt::format;
#[allow(warnings)]
use bracket_lib::prelude::*; // use everything from bracket-lib exported within the prelude

enum GameMode {
    Menu,
    Playing,
    End,
}

// Constants - known at compile time
const SCREEN_WIDTH : i32 = 80;
const SCREEN_HEIGHT : i32 = 50;
const FRAME_DURATION :  f32 = 75.0;  // in milliseconds
const TERMINAL_VELOCITY: f32 = 2.0;
const GRAVITY: f32 = 0.2;
const FLAP_STRENGTH: f32 = 1.0;

struct Obstacle {
    x: i32,
    gap_y: i32,
    size: i32
}

impl Obstacle {
    fn new(x: i32, score: i32) -> Self {
        let mut random = RandomNumberGenerator::new();
        Self {
            x,
            gap_y: random.range(10, 40),
            size: i32::max(2, 20 - score)
        }
    }


    fn render(&mut self, ctx: &mut BTerm, player_x: i32){

        let screen_x = self.x - player_x;
        let half_size = self.size / 2;

        // Draw top half of the obstacle
        for y in 0..self.gap_y - half_size {
            ctx.set(screen_x, y, RED, BLACK, to_cp437('|'));
        }

        // Draw bottom half of the obstacle
        for y in self.gap_y + half_size..SCREEN_HEIGHT {
            ctx.set(screen_x, y, RED, BLACK, to_cp437('|'));
        }
    }

    fn hit_obstacle(&self, player: &Player) -> bool {
        let half_size = self.size / 2;
        let does_x_match = player.x == self.x; // possible collision if at same x coordinate
        let player_above_gap = (player.y as i32) < self.gap_y - half_size; // compare with upper gap
        let player_below_gap = (player.y as i32) > self.gap_y + half_size; // compare with lower gap

        // return true if collision occurs
        does_x_match && (player_above_gap || player_below_gap)
    }
}

// The Dragons current state
struct Player {
    x: i32,     // world space location in terminal characters, represents progress through level
    y: f32,     // vertical position in screen space
    velocity: f32   // players vertical velocity
}
impl Player {

    fn new(x: i32, y: i32) -> Self {
        Self {
            x,
            y: y as f32,
            velocity: 0.0,
        }
    }

    fn render(&mut self, ctx: &mut BTerm){ // allow to mutate instance and pass in context for game engine
        // sets a single character on the screen
        // this is screen space, world space is defined by values in player.x and player.y
        ctx.set(0, self.y as i32, YELLOW, BLACK, to_cp437('@'));
    }

    fn gravity_and_move(&mut self){
        // NOTE: +ve velocity is in downwards screen direction, i.e. +ve Y co-ordinate direction
        if self.velocity < TERMINAL_VELOCITY {
            self.velocity += GRAVITY; // increase gravity if velocity if less than 2.0
        }

        self.y += self.velocity; // cast to i32 and decrement y (casting rounds down)

        self.x += 1;  // move horizontally across the screen

        if self.y < 0.0 { // zero is the 'top' of the screen
            self.y = 0.0;  // y can never be less than zero
        }

    }

    fn flap(&mut self){
        self.velocity = -FLAP_STRENGTH; // velocity in upwards direction
    }
}

// Games State
struct State {
    mode: GameMode,  // store current game mode
    player: Player,      // players instance object
    frame_time: f32,     // time accumulated between frames, used to control game speed
    score: i32, // players current score
    obstacle: Obstacle,
}

impl State {

    fn new() -> Self {
        Self {
            mode: GameMode::Menu,  // initial State
            frame_time: 0.0,
            player: Player::new(5, 25),
            obstacle: Obstacle::new(SCREEN_WIDTH, 0),
            score: 0,
        }
    }

    fn restart(&mut self){ // reset to initial state
        self.player.x = 5;
        self.player.y = 25.0;
        self.frame_time = 0.0;
        self.mode = GameMode::Playing;
        self.obstacle = Obstacle::new(SCREEN_WIDTH  ,0);
        self.score = 0;
    }

    fn main_menu(&mut self, ctx: &mut BTerm){
        // clear screen
        ctx.cls();

        // Print Menu Options
        ctx.print_centered(5, "Welcome to Flappy Dragon");
        ctx.print_centered(8, "(P) Play Game");
        ctx.print_centered(9, "(Q) Quit Game");

        if let Some(key) = ctx.key { // run block if key is pressed

            match key { // match key pressed to an action
                VirtualKeyCode::P => self.restart(), // resets state and changes mode to Playing
                VirtualKeyCode::Q => ctx.quitting = true, // instruct bracket-lib to terminate program
                _ => {} // all other keys do nothing
            }

        }
    }

    fn dead(&mut self, ctx: &mut BTerm){
        ctx.cls();
        ctx.print_centered(5, "You are dead!");
        ctx.print_centered(6, &format!("You earned {} points", self.score));
        ctx.print_centered(8, "(P) Play Again");
        ctx.print_centered(9, "(Q) Quit Game");

        if let Some(key) = ctx.key {

            match key {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => ctx.quitting = true, // terminate program
                _ => {} // all other keys do nothing
            }
        }
    }

    fn play(&mut self, ctx: &mut BTerm){
        // Logic for Play
        ctx.cls_bg(NAVY); // clear screen and change background color

        // increment frame time so we have an idea of game speed
        // frame_time_ms is time elapsed since tick was last called
        self.frame_time += ctx.frame_time_ms;

        if self.frame_time > FRAME_DURATION {
            self.frame_time = 0.0; // reset to zero

            // Run physics simulation
            // at the end of a frame, execute gravity effect. increases y and increases x
            self.player.gravity_and_move();
        }

        // if space bar is pressed, flap wings - decreases y
        if let Some(VirtualKeyCode::Space) = ctx.key {
            self.player.flap();
        }

        // render the updated players state
        self.player.render(ctx);

        ctx.print(0, 0, "Press SPACE to flap.");
        ctx.print(0, 1, &format!("Score: {}", self.score));

        self.obstacle.render(ctx, self.player.x);

        if self.player.x > self.obstacle.x {
            self.score += 1;
            self.obstacle = Obstacle::new(self.player.x + SCREEN_WIDTH, self.score);
        }

        // if we have fallen off bottom of screen or hit an obstacle
        if (self.player.y as i32) > SCREEN_HEIGHT || self.obstacle.hit_obstacle(&self.player){
            // transition to End State
            self.mode = GameMode::End;
        }
    }
}


// State now implements the trait / interface for GameState
impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        // &mut self allows tick function to access and change your State instance
        // ctx provides a window into the currently running bracket-terminal
        // ctx provides functions for interacting with the game display
        // ctx.cls(); // clear the screen
        // ctx.print(1, 1, "Hello, Bracket Terminal!"); // print to game window
        match self.mode {
            GameMode::Menu => self.main_menu(ctx),
            GameMode::End => self.dead(ctx),
            GameMode::Playing => self.play(ctx),
        }
    }
}

fn main() -> BError { // BError is a Result type
    // initialise game engine
    // describe type of window to use and game loop to create
    // setup via calling .build()
    let context = BTermBuilder::simple80x50()
        .with_title("Flappy Dragon")
        .build()?; // use '?' to pass errors to the parent function

    // link context to State that implements a tick function
    main_loop(context, State::new())
}
