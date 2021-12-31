use std::io::{stdout, Stdout, Write};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;

use crossterm::cursor::{Hide, MoveTo};
use crossterm::event::{Event, KeyCode, poll, read};
use crossterm::{ExecutableCommand, execute};
use crossterm::style::{Color, Print, ResetColor, SetForegroundColor};
use crossterm::terminal::{Clear, ClearType};
use rand;
use rand::Rng;

fn main() {
    let (tx, rx): (Sender<KeyCode>, Receiver<KeyCode>) = mpsc::channel();

    thread::spawn(move || {
        read_user_input(tx);
    });

    let (cols, rows) = (30, 20);
    let mut out = stdout();
    execute!(out, Clear(ClearType::All), Hide);

    let mut snake: Snake = build_snake();
    let mut fruit: Point = build_fruit(cols, rows);
    let cage = build_cage(cols, rows);

    display_cage(&out, cage);

    loop {
        thread::sleep(Duration::from_millis(100));
        let result = rx.try_recv();
        if result.is_ok() {
            snake.set_direction(result.unwrap());
        }

        // se mangio il frutto, ne creo uno nuovo random:
        if snake.move_snake(&fruit) {
            fruit = build_fruit(cols, rows);
        }

        if snake.is_dead(cols as i16, rows as i16) {
            execute!(out, SetForegroundColor(Color::Red), MoveTo(cols, rows));
            println!("\nGAME OVER. SCORE: {}", snake.body.len() - 4);
            break;
        }

        execute!(out, Hide);
        execute!(out, SetForegroundColor(Color::Green));
        display_snake(&out, &snake);

        execute!(out, SetForegroundColor(Color::Red));
        display_point(&out, &fruit);

        execute!(out, Hide);
        out.flush();
    }

    execute!(out, ResetColor);
    out.flush();
}

fn build_fruit(cols: u16, rows: u16) -> Point {
    return Point {
        x: rand::thread_rng().gen_range(5..(cols - 5)) as i16,
        y: rand::thread_rng().gen_range(5..(rows - 5)) as i16,
        char: 'X',
    };
}

fn display_snake(mut out: &Stdout, snake: &Snake) {
    execute!(out, SetForegroundColor(Color::Green));
    for p in &snake.body {
        display_point(out, p);
    }
}

fn display_point(mut out: &std::io::Stdout, p: &Point) {
    execute!(out, MoveTo(p.x as u16, p.y as u16), Print(p.char));
}

fn build_cage(cols: u16, rows: u16) -> Vec<Point> {
    let mut cage: Vec<Point> = vec![];
    for c in 0..cols {
        cage.push(Point {
            x: c as i16,
            y: 0,
            char: '-',
        });
        cage.push(Point {
            x: c as i16,
            y: (rows - 1) as i16,
            char: '-',
        });
    }

    for r in 0..rows {
        cage.push(Point {
            x: 0,
            y: r as i16,
            char: '|',
        });
        cage.push(Point {
            x: (cols - 1) as i16,
            y: r as i16,
            char: '|',
        });
    }
    return cage;
}

fn display_cage(mut out: &Stdout, cage: Vec<Point>) {
    execute!(out, SetForegroundColor(Color::Blue));
    for p in &cage {
        display_point(out, p);
    }
}

struct Point {
    x: i16,
    y: i16,
    char: char,
}

#[derive(Debug)]
struct Direction {
    x: i8,
    y: i8,
}

const DIRECTION_UP: Direction = Direction { x: 0, y: -1 };
const DIRECTION_DOWN: Direction = Direction { x: 0, y: 1 };
const DIRECTION_LEFT: Direction = Direction { x: -1, y: 0 };
const DIRECTION_RIGHT: Direction = Direction { x: 1, y: 0 };
const DIRECTION_NONE: Direction = Direction { x: 0, y: 0 };

struct Snake {
    body: Vec<Point>,
    direction: Direction,
}

fn build_snake() -> Snake {
    return Snake {
        body: vec![Point { x: 1, y: 10, char: 'O' },
                   Point { x: 1, y: 11, char: 'O' },
                   Point { x: 1, y: 12, char: 'O' },
                   Point { x: 1, y: 11, char: ' ' }],
        direction: DIRECTION_UP,
    };
}

impl PartialEq for Direction {
    fn eq(&self, other: &Self) -> bool {
        return self.x == other.x && self.y == other.y;
    }
}

impl Snake {

    fn is_dead(&self, cols:i16, rows:i16) -> bool {
        let head = &self.body[0];
        if head.x >= cols || head.y >= rows || head.x < 0 || head.y < 0 {
            return true;
        }

        for b in 1..(self.body.len()) {
            if self.body[b].x == head.x && self.body[b].y == head.y {
                return true;
            }
        }

        return false;
    }

    fn set_direction(&mut self, key_code: KeyCode) {
        let direction = self.match_direction(key_code);
        if direction == DIRECTION_NONE {
            return;
        }

        if self.direction.x == 0 && self.direction.x != direction.x {
            self.direction = direction;
        } else if self.direction.y == 0 && self.direction.y != direction.y {
            self.direction = direction;
        }
    }

    fn match_direction(&mut self, key_code: KeyCode) -> Direction {
        match key_code {
            KeyCode::Up => DIRECTION_UP,
            KeyCode::Down => DIRECTION_DOWN,
            KeyCode::Left => DIRECTION_LEFT,
            KeyCode::Right => DIRECTION_RIGHT,
            _ => DIRECTION_NONE
        }
    }

    /**
     * moves the snake one place in the direction
     */
    fn move_snake(&mut self, fruit: &Point) -> bool {
        let eat = self.eat(fruit);

        // not eating the fruit:
        if !eat {
            // remove the last item:
            let len = self.body.len();
            self.body.pop();
            self.body[len - 2].char = ' ';
        }

        // get the first position
        let head = &self.body[0];

        // add the new position
        let new_head = Point {
            x: head.x + (self.direction.x as i16),
            y: head.y + (self.direction.y as i16),
            char: 'O',
        };

        self.body.insert(0, new_head);

        return eat;
    }

    fn eat(&self, fruit: &Point) -> bool {
        // get the first position
        let head = &self.body[0];
        return head.y == fruit.y && head.x == fruit.x;
    }
}

fn read_user_input(tx: Sender<KeyCode>) {
    loop {
        // `poll()` waits for an `Event` for a given time period
        if poll(Duration::from_millis(500)).unwrap() {
            // It's guaranteed that the `read()` won't block when the `poll()`
            // function returns `true`
            match read().unwrap() {
                Event::Key(event) => tx.send(event.code).unwrap(),
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::KeyCode;
    use rand::Rng;

    use crate::{build_snake, DIRECTION_LEFT, DIRECTION_RIGHT, DIRECTION_UP};

    #[test]
    fn change_direction() {
        let mut snake = build_snake();
        snake.set_direction(KeyCode::Up);
        assert_eq!(snake.direction, DIRECTION_UP);

        snake.set_direction(KeyCode::Down);
        assert_eq!(snake.direction, DIRECTION_UP);

        snake.set_direction(KeyCode::Left);
        assert_eq!(snake.direction, DIRECTION_LEFT);
        snake.set_direction(KeyCode::Left);
        assert_eq!(snake.direction, DIRECTION_LEFT);
        snake.set_direction(KeyCode::Right);
        assert_eq!(snake.direction, DIRECTION_LEFT);
    }
}