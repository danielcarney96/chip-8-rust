use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

pub struct Display {
    pub context: sdl2::Sdl,
    canvas: Canvas<Window>,
}

impl Display {
    pub fn new() -> Self {
        let context = sdl2::init().unwrap();
        let canvas = create_canvas(&context);

        Display {
            context: context,
            canvas: canvas,
        }
    }

    pub fn draw(&mut self, pixels: &[[u8; 64]; 32]) {
        for (y, row) in pixels.iter().enumerate() {
            for (x, &col) in row.iter().enumerate() {
                let x = (x as u32) * 20;
                let y = (y as u32) * 20;

                self.canvas.set_draw_color(black_or_green(col));
                let _ = self.canvas.fill_rect(Rect::new(x as i32, y as i32, 20, 20));
            }
        }
        self.canvas.present();
    }
}

fn create_canvas(context: &sdl2::Sdl) -> Canvas<Window> {
    let video_subsystem = context.video().unwrap();
    let window = video_subsystem
        .window("CHIP-8 Interpreter", 64 * 20, 32 * 20)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    canvas
}

fn black_or_green(value: u8) -> Color {
    if value == 0 {
        Color::RGB(0, 0, 0)
    } else {
        Color::RGB(0, 250, 0)
    }
}
