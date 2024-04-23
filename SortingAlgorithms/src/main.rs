/*****************************************************************/
//! [Mandelbrot Set Zoom]
/*****************************************************************/
//!
//! This is a program which handles the simulation and visual 
//! rendering of a popular mathematical fractal called The 
//! Mandelbrot Set. The set is defined by the points on the
//! complex plane which converge on the function z^2 + c.
//! 
//! This program uses the Piston game crate and OpenGL on
//! the backend to perform all the rendering for the zoom.
//! For the sake of parallelizing the code, Rayon was used,
//! and after brief testing a near linear speedup was observed.
//!
//! [Authors]
//! Aiden Manuel (Original programming and idea),
//! Matthew Peterson (Parallel programming and optimizations, commenting)
//!
//! [Class] CS 3123, Dr. Jeff Mark McNally
//!
//! [Date] Submitted April 11, 2024
/*****************************************************************/

// Define external libraries.
extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate rand;
extern crate chrono;
extern crate rayon;

// Import necessary functions from external libraries.
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;
use piston::GenericEvent;
use graphics::rectangle::centered;
use rand::prelude::*;

// All metrics pre-defined as constants
// so that they can be used to define
// array sizes.
const SCREEN_WIDTH: f64 = 1000.0;
const SCREEN_HEIGHT: f64 = 700.0;

const NUM_COLS: i32 = 100;



/// [App]
/// The App struct defines the Piston application and associated
/// data. All fields within this structure are statically accessible
/// from within the application's associated methods.
pub struct App { 
    // OpenGL drawing backend.
    gl: GlGraphics,
    paused: bool,
    doTick: bool,
    columns: Vec<i32>, 
    choice: i32,
    pointer: usize,
    bubble_completed: i32,
    num_cols: i32,
}

/// [App]
/// Application related methods.
impl App {
    
    /// [Render]
    /// The render method is required by Piston in order to service
    /// the application control-flow, using callbacks. The render
    /// method is specifically meant to be where all calls to OpenGL
    /// happen, and is meant to be called every frame.
    ///
    /// This program implements the render method by checking the current
    /// value in vals at each pixel and then colouring it based on the scalar
    ///
    /// Being a Piston callback, its only parameters are itself,
    /// and the Piston render arguments.

    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        // Variables for colouring:
        let mut r: f32;
        let mut g: f32;
        let mut b: f32;
        let mut diff_r: i32;
        let mut diff_b: i32;
        let mut column: [f32; 4];
        let background: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

        // Variables for column position
        let mut col_height: f64;
        let col_width: f64 = (SCREEN_WIDTH / self.num_cols as f64) * 0.5;
        let mut x: f64;
        let mut y: f64;

        // The following block of code will overwrite the OpenGL window with background colour.
        self.gl.draw(args.viewport(), |c, gl| {
            // Create the necessary components to draw with:
            let background_fill =
                rectangle::rectangle_by_corners(0.0, 0.0, SCREEN_WIDTH as f64, SCREEN_HEIGHT as f64);
            let transform = c.transform;

            // Collect all components and write to the screen.
            rectangle(background, background_fill, transform, gl);
        });

        for i in 0..self.num_cols {

            // Handling Column Position
            col_height = (self.columns[i as usize] as f64 / self.num_cols as f64) * (SCREEN_HEIGHT * 0.75) * 0.5;

            x = (i as f64 * col_width * 2.0) + col_width;
            y = SCREEN_HEIGHT - col_height;
    
            let square = centered([x, y, col_width - 0.5, col_height]);

            // Handling Column Colour
            if self.columns[i as usize] > self.num_cols / 2{
                diff_r = 0;
                diff_b = self.columns[i as usize] - self.num_cols / 2;
            } else {
                diff_r = self.num_cols / 2 - self.columns[i as usize];
                diff_b = 0;
            }

            r = diff_r as f32 / (self.num_cols as f32 / 2.0);
            if self.columns[i as usize] <= self.num_cols / 2 {g = self.columns[i as usize] as f32 / (self.num_cols as f32 / 2.0);} 
            else {g = (self.num_cols - self.columns[i as usize]) as f32 / (self.num_cols as f32 / 2.0);} 
            b = diff_b as f32 / (self.num_cols as f32 / 2.0);

            column = [r, g, b, 1.0];
            
            // OpenGL is used for rendering it to the screen.
            self.gl.draw(args.viewport(), |c, gl| {

                let transform = c
                    .transform;

                rectangle(column, square, transform, gl);
            });
        }
    }


    /// [Update]
    ///
    /// The update method is required by Piston in order to service
    /// the application logic (as opposed to rendering) using callbacks.
    /// The update method contains user-defined logic which does not
    /// necessarily have to do with drawing to OpenGL.
    
    fn update(&mut self, _args: &UpdateArgs) {
        if !self.paused || self.doTick{
            // Pick Sorting Algorithm

            match self.choice.abs(){ 
                0=>println!("Selection"), 
                1=>println!("Insertion"), 
                2=>self.bubble_step(), 
                _=>println!("{}", self.choice),
            }

            // Call Sorting Algorithm

            if self.bubble_completed < (self.columns.len() - 2) as i32 && self.pointer == 0 {
                self.bubble_completed += 1;
            }


            // Done

            self.doTick = !self.doTick;
        }
    }
    

    /// [Event]
    ///
    /// The event method is required by Piston in order to service
    /// user interaction using callbacks. This includes key presses,
    /// and support for mouse interaction. Such input is necessary
    /// for clearing the board, regenerating the board, and drawing
    /// directly to the board.
    /// 
    fn event<E: GenericEvent>(&mut self, e: &E) {
        use piston::input::{Button, Key};

        // Key Functions Added!
        // Space:   pause the simulation
        // P:       print the current information
        if let Some(Button::Keyboard(key)) = e.press_args() {
                match key {
                    Key::Space => {self.paused = !self.paused; if self.paused { println!("paused") } else { println!("playing") };},
                    Key::W => self.doTick = true,
                    Key::R => self.randomize(),
                    Key::Right => (self.choice, self.pointer, self.bubble_completed) = ((self.choice + 1) % 3, 0, 0),
                    Key::Left => (self.choice, self.pointer, self.bubble_completed) = ((self.choice - 1) % 3, 0, 0),
                    Key::NumPadPlus => {self.num_cols += 1; self.columns.push((self.columns.len() + 1) as i32);},
                    Key::NumPadMinus => {self.num_cols -= 1; self.columns.pop();},
                    _ => {}
            }
        }
    }

    fn randomize(&mut self) {
        (self.pointer, self.bubble_completed) = (0, 0);

        let mut temp:Vec<i32> = vec![];
        let mut length;
        let mut rng = rand::thread_rng();

        for _i in 0..self.num_cols{
            length = self.columns.len();
            let rand: f64 = rng.gen();
            let index = (rand * length as f64) as usize;
            temp.push(self.columns.swap_remove(index));
        }

        self.columns = temp;
    }

    fn bubble_step(&mut self) {
        let i = self.pointer;
        let j = i + 1;

        if self.columns[i] > self.columns[j] {
            let temp = self.columns[i];
            self.columns[i] = self.columns[j];
            self.columns[j] = temp;
        }

        if self.pointer < self.columns.len() - 2 - self.bubble_completed as usize {
            self.pointer += 1;
        } else {
            self.pointer = 0;
        }
    }
}

/// [Main]
///
/// Note: Most of this main method comes from a Piston tutorial.
/// https://github.com/PistonDevelopers/Piston-Tutorials/tree/master/getting-started
///
/// This method sets up the application state, and initializes the OpenGL backend for
/// execution by Piston.

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create a Glutin window.
    let mut window: Window = WindowSettings::new("Sorting Algorithms", [SCREEN_WIDTH, SCREEN_HEIGHT])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut columns = vec![];

    for i in 1..NUM_COLS + 1 {
        columns.push(i);
    }

    // Create a new simulation, and run it
    let mut app = App {
        gl: GlGraphics::new(opengl),
        paused: false,
        doTick: false,
        columns: columns,
        choice: 0,
        pointer: 0,
        bubble_completed: 0,
        num_cols: NUM_COLS,
    };

    // The main piston loop, which actually runs all the app
    // functions repeatedly
    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        app.event(&e);

        if let Some(args) = e.render_args() {
            app.render(&args);
        }

        if let Some(args) = e.update_args() {
            app.update(&args);
        }
    }
}
