use std::ops::Range;
use sdl2::{self, pixels::Color, event::Event, keyboard::Keycode, mouse::MouseButton, render::Canvas, rect::{Rect, Point}};
use ahash::HashSetExt;

struct Conditions
{
    underpop: Range<u8>, //underpopulation, must start at 0. Will die in this range
    same    : Range<u8>, //must start at the upper excluded limit of underpop. Will not change state in this range
    alive   : Range<u8>, //must start the upper excluded limit of same. Will be alive in this range
    overpop : Range<u8>  //overpopulation, must start the upper excluded limit of alive and end at 10. Will die in this range
}

impl std::default::Default for Conditions
{
    fn default() -> Self
    {
        Self 
        {
            underpop: 0..2,
            same: 2..3,
            alive: 3..4,
            overpop: 4..10
        }
    }
}

impl Conditions
{
    fn check_valid(&self) -> bool
    {
        self.underpop.start == 0 &&
        self.underpop.end == self.same.start &&
        self.same.end == self.alive.start &&
        self.alive.end == self.overpop.start &&
        self.overpop.end == 10
    }

    fn cell_next_state(&self, neighbours: u8, is_alive: bool) -> bool
    {
        self.alive.contains(&neighbours) || (is_alive && self.same.contains(&neighbours))
    }
}

fn main() {
    let mut res: [u32; 2] = [600, 600];
    let mut start_coords: [i32; 2] = [0, 0];
    let mut pixel_size: [u32; 2] = [3, 3];
    let (mut canvas, mut event_pump) = init_canvas(res);

    let mut target_frame_ms: f64 = 13.;
    let mut target_frame_time = std::time::Duration::from_micros((target_frame_ms * 1000.) as u64);

    let mut grid_lines = false;

    let mut alive_cells: ahash::HashSet<[i32; 2]> = HashSetExt::new();
    let cond = Conditions::default();
    if !cond.check_valid() {panic!("the provided conditions weren't valid");};
    
    let mut lmd_down = false;
    let mut mmd_down = false;

    let mut pause = false;

    let mut last_click_grid_cell = start_coords;

    'running: loop
    {
        let mut changed = false;

        let start_time = std::time::SystemTime::now();
        for event in event_pump.poll_iter()
        {
            match event
            {
                Event::Quit {..} => {break 'running}
                
                Event::KeyDown {keycode, ..} =>
                {
                    match keycode.unwrap()
                    {
                        Keycode::Space => {pause = !pause;}
                        Keycode::Left =>
                        {
                            target_frame_ms -= 2.;
                            target_frame_time = std::time::Duration::from_micros((target_frame_ms * 1000.) as u64);
                        }
                        Keycode::Right =>
                        {
                            target_frame_ms += 2.;
                            target_frame_time = std::time::Duration::from_micros((target_frame_ms * 1000.) as u64);
                        }
                        Keycode::Up =>
                        {
                            alive_cells = next_grid_state(alive_cells, &cond);
                            changed = true;
                        }
                        Keycode::G =>
                        {
                            grid_lines = !grid_lines;
                            changed = true;
                        }
                        _ => {}
                    }
                }

                Event::KeyUp {keymod, ..} =>
                {
                    match keymod
                    {
                        _ => {}
                    }
                }

                Event::MouseButtonDown {mouse_btn, x, y, ..} =>
                {
                    match mouse_btn
                    {
                        MouseButton::Left =>
                        {
                            lmd_down = true;
                            last_click_grid_cell = mouse_to_grid_coords([x as u32, y as u32], pixel_size, start_coords);
                        }
                        MouseButton::Middle =>
                        {
                            mmd_down = true;
                            let coords = mouse_to_grid_coords([x as u32, y as u32], pixel_size, start_coords);
                            if alive_cells.contains(&coords)
                            {
                                alive_cells.remove(&coords);
                            }
                            else
                            {
                                alive_cells.insert(coords);
                            }
                            changed = true;
                        }
                        MouseButton::Right =>
                        {
                            spawn_glider(mouse_to_grid_coords([x as u32, y as u32], pixel_size, start_coords), &mut alive_cells);
                            changed = true;
                        }
                        _ => {}
                    }
                }

                Event::MouseButtonUp {mouse_btn, ..} =>
                {
                    match mouse_btn
                    {
                        MouseButton::Left =>
                        {
                            lmd_down = false;
                        }
                        MouseButton::Middle =>
                        {
                            mmd_down = false;
                        }
                        _ => {}
                    }
                }

                Event::MouseMotion {x, y, ..} =>
                {
                    if mmd_down
                    {
                        let coords = mouse_to_grid_coords([x as u32, y as u32], pixel_size, start_coords);
                        if !alive_cells.contains(&coords)
                        {
                            alive_cells.insert(coords);
                            changed = true;
                        }
                    }
                    if lmd_down
                    {
                        let coords = mouse_to_grid_coords([x as u32, y as u32], pixel_size, start_coords);
                        start_coords[0]+= last_click_grid_cell[0] - coords[0];
                        start_coords[1]+= last_click_grid_cell[1] - coords[1];

                        changed = true;
                    }
                }

                Event::MouseWheel {y, ..} =>
                {
                    let old_grid_size = [res[0] / pixel_size[0], res[1] / pixel_size[1]];

                    pixel_size[0] = if pixel_size[0] as i32 + y > 1 {(pixel_size[0] as i32 + y) as u32} else {1};
                    pixel_size[1] = if pixel_size[1] as i32 + y > 1 {(pixel_size[1] as i32 + y) as u32} else {1};

                    let new_grid_size = [res[0] / pixel_size[0], res[1] / pixel_size[1]];

                    start_coords[0] += (old_grid_size[0] as i32 - new_grid_size[0] as i32) / 2;
                    start_coords[1] += (old_grid_size[1] as i32 - new_grid_size[1] as i32) / 2;

                    changed = true;
                }

                Event::Window {win_event, ..} =>
                {
                    match win_event
                    {
                        sdl2::event::WindowEvent::Resized(x, y) =>
                        {
                            canvas.window_mut().set_size(x as u32, y as u32).unwrap();
                            res = [x as u32, y as u32];
                            changed = true;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        if !pause
        {
            alive_cells = next_grid_state(alive_cells, &cond);
            changed = true;
        }

        if changed
        {
            draw_grid(&alive_cells, &mut canvas, start_coords, pixel_size, 
                (grid_lines || (pixel_size[0] > 20 && pixel_size[1] > 20)) && (pixel_size[0] > 1 && pixel_size[1] > 1)
                );
 
            if !pause
            {
                let sleep_time = target_frame_time.saturating_sub(
                    std::time::SystemTime::now().duration_since(start_time).unwrap()
                );
                std::thread::sleep(
                    sleep_time
                );
            }
        }
    }

}

fn get_nbghr_nb([x, y]: [i32; 2], alive_cells: &ahash::HashSet<[i32; 2]>) -> u8
{
    let mut nb: u8 = 0;

    let nghbrs = [
        [x-1, y-1], [x-1, y], [x-1, y+1],
        [x, y-1],             [x, y+1],
        [x+1, y-1], [x+1, y], [x+1, y+1]
    ];

    for cell in nghbrs
    {
        if alive_cells.contains(&cell)
        {
            nb+=1;
        }
    }
    
    nb
}

fn spawn_glider(glider_coords: [i32; 2], alive_cells: &mut ahash::HashSet<[i32; 2]>)
{

    let glider = 
    [
        [0, 0], [2, 0], [1, 1], [2, 1], [1, 2]
    ];

    for cell in glider
    {
        if !alive_cells.contains(&cell)
        {
            alive_cells.insert([glider_coords[0] + cell[0], glider_coords[1] + cell[1]]);
        }
    }
    
}

fn draw_grid(alive_cells: &ahash::HashSet<[i32; 2]>, canvas: &mut Canvas<sdl2::video::Window>, start_coords: [i32; 2], pixel_size: [u32; 2], grid_lines: bool)
{
    canvas.set_draw_color(Color::WHITE);
    canvas.clear();

    canvas.set_draw_color(Color::BLACK);

    for coord in alive_cells.iter()
    {
        let [x, y] = coord;
        canvas.fill_rect(Rect::new(
            pixel_size[0] as i32 * (x - start_coords[0]),
            pixel_size[1] as i32 * (y - start_coords[1]),
            pixel_size[0],
            pixel_size[1]
        )).unwrap();
    }

    if grid_lines
    {
        let (size_x, size_y) = canvas.output_size().unwrap();

        canvas.set_draw_color(Color::GRAY);

        for x in 0..(size_x / pixel_size[0] +1)
        {
            let line_x = (x * pixel_size[0]) as i32;
            canvas.draw_line(Point::new(line_x, 0), Point::new(line_x, size_y as i32)).unwrap();
        }
        for y in 0..(size_y / pixel_size[1] +1)
        {
            let line_y = (y * pixel_size[1]) as i32;
            canvas.draw_line(Point::new(0, line_y), Point::new(size_x as i32, line_y)).unwrap();
        }
    }

    canvas.present();
}

fn mouse_to_grid_coords(mouse: [u32; 2], pixel_size: [u32; 2], start: [i32; 2]) -> [i32; 2]
{
    [
        start[0] + mouse[0] as i32 / pixel_size[0] as i32,
        start[1] + mouse[1] as i32 / pixel_size[1] as i32
    ]
}

fn next_grid_state(alive_cells: ahash::HashSet<[i32; 2]>, cond: &Conditions) -> ahash::HashSet<[i32; 2]>
{
    let mut next_state: ahash::HashSet<[i32; 2]> = HashSetExt::new();
    let mut checked: ahash::HashSet<[i32; 2]> = HashSetExt::new();

    for coord in alive_cells.iter()
    {
        let [x, y]: [i32; 2] = *coord;

        let nghbrs = [
            [x-1, y-1], [x-1, y], [x-1, y+1],
            [x, y-1],   [x, y],   [x, y+1],
            [x+1, y-1], [x+1, y], [x+1, y+1]
        ];

        for cell in nghbrs
        {
            if !checked.contains(&cell) && cond.cell_next_state(get_nbghr_nb(cell, &alive_cells), alive_cells.contains(&cell))
            {
                next_state.insert(cell);
                checked.insert(cell);
            }
        }
    }

    next_state
}


fn init_canvas(res: [u32; 2]) -> (sdl2::render::Canvas<sdl2::video::Window>, sdl2::EventPump)
{
    let sdl_instance = sdl2::init().unwrap();
    let video = sdl_instance.video().unwrap();

    let window = video.window("game of life", res[0], res[1])
        .position_centered()
        .resizable()
        .build()
        .unwrap();

    let canvas = window.into_canvas()
        .build()
        .unwrap();

    let events = sdl_instance.event_pump().unwrap();

    (canvas, events)
}