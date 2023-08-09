use ncurses::*;
use rand::Rng;
use std::collections::vec_deque::VecDeque;

const FPS: u32 = 120;

const PIPE_WIDTH: i32 = 10;
const PIPE_GAP: i32 = 12;

const HOLE_HEIGHT: i32 = 7;

struct GameState {
    bird: Bird,

    pipes: VecDeque<Pipe>,

    should_quit: bool,
}

#[derive(Debug)]
struct Pipe {
    x_start: f32,

    hole_y_start: f32,
}

impl Pipe {
    fn new(x: i32) -> Pipe {
        Pipe {
            x_start: x as f32,
            hole_y_start: rand::thread_rng().gen_range(10..20) as f32,
        }
    }
}

struct Bird {
    x: f32,
    y: f32,

    velocity: f32,
}

enum Action {
    Jump,
    Quit,
}

fn init() -> (WINDOW, GameState) {
    initscr();
    noecho();
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
    nodelay(stdscr(), true);

    let win = newwin(LINES(), COLS(), 0, 0);

    let initial_state = GameState {
        bird: Bird {
            x: COLS() as f32 / 8.0,
            y: LINES() as f32 / 2.5,
            velocity: 0.2,
        },

        pipes: (1..20)
            .map(|i| Pipe::new((COLS() / 2) + (i * (PIPE_WIDTH as i32 + PIPE_GAP))))
            .collect(),

        should_quit: false,
    };

    (win, initial_state)
}

fn handle_input() -> Option<Action> {
    getch()
        .try_into()
        .ok()
        .and_then(char::from_u32)
        .map(|c| match c {
            ' ' => Some(Action::Jump),
            'q' => Some(Action::Quit),
            _ => None,
        })
        .flatten()
}

fn update(state: GameState, action: Option<Action>) -> GameState {
    use Action::*;

    let mut state = state;

    if let Some(Quit) = action {
        state.should_quit = true;
    }

    let hit_pipe = state.pipes.iter().any(|pipe| {
        (pipe.x_start as i32..(pipe.x_start as i32 + PIPE_WIDTH)).contains(&(state.bird.x as i32))
            && !(pipe.hole_y_start as i32..(pipe.hole_y_start as i32 + HOLE_HEIGHT))
                .contains(&(state.bird.y as i32))
    });

    if hit_pipe {
        return state;
    }

    if let Some(Jump) = action {
        state.bird.velocity = -0.2;
    }

    state.pipes = state
        .pipes
        .into_iter()
        .map(|pipe| {
            let mut pipe = pipe;
            pipe.x_start -= 0.2;

            pipe
        })
        .collect();

    if (state.pipes.get(0).unwrap().x_start + PIPE_WIDTH as f32) < 0.0 {
        state.pipes.pop_front();
        state.pipes.push_back(Pipe::new(
            state.pipes.get(state.pipes.len() - 1).unwrap().x_start as i32 + PIPE_WIDTH + PIPE_GAP,
        ))
    }

    state.bird.y = state.bird.y + state.bird.velocity;
    state.bird.velocity += 0.005;

    state
}

fn render(win: WINDOW, state: &GameState) {
    werase(win);
    for y in 0..LINES() {
        for x in 0..COLS() {
            mvwaddch(win, y as i32, x as i32, ' ' as u32);
        }
    }

    for pipe in &state.pipes {
        for y in 0..LINES() {
            for x in (pipe.x_start as i32)..(pipe.x_start as i32 + PIPE_WIDTH) {
                if y < pipe.hole_y_start as i32 || y > pipe.hole_y_start as i32 + HOLE_HEIGHT {
                    mvwaddch(win, y as i32, x as i32, '#' as u32);
                }
            }
        }
    }

    let bird_char = if state.bird.velocity < 0.0 { 'p' } else { 'b' };

    mvwaddch(
        win,
        state.bird.y as i32,
        state.bird.x as i32,
        bird_char.into(),
    );

    wrefresh(win);
}

fn main() {
    let (win, initial_state) = init();

    let mut state = initial_state;
    loop {
        if state.should_quit {
            break;
        }

        let maybe_action = handle_input();

        state = update(state, maybe_action);

        render(win, &state);

        std::thread::sleep(std::time::Duration::from_millis((1000 / FPS).into()));
    }

    endwin();
}
