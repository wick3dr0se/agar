use macroquad::{prelude::*, ui::root_ui};

struct Cell {
    center: Vec2,
    radius: f32,
    absorbed: bool
}

impl Cell {
    fn draw_radius(&self, color: Color) {
        draw_circle(self.center.x, self.center.y, self.radius, color);

        let text = &(self.radius as i32).to_string();
        let text_dims = measure_text(text, None, self.radius as u16, 1.0);
        let text_pos: Vec2 = self.center - vec2(text_dims.width, -text_dims.height) / 2.0;
    
        draw_text(text, text_pos.x, text_pos.y, self.radius, BLACK);
    }

    fn intersects(&self, cell: &Cell) -> bool {
        let radius_sum = (self.radius + cell.radius).powi(2);

        self.center.distance_squared(cell.center) <= radius_sum
    }

    fn try_absorb(&mut self, cell: &mut Cell) -> Option<f32> {
        if self.intersects(&cell) && self.radius > cell.radius {
            cell.absorbed = true;
            
            let growth = self.radius - (self.radius.powi(2) + cell.radius.powi(2)).sqrt();
            self.radius -= growth;

            return Some(growth * 0.1)
        }

        None
    }
}

struct Food {
    cell: Cell
}

impl Food {
    fn random(bounds: Vec2) -> Self {
        Self {
            cell: Cell {
                center: vec2(
                    rand::gen_range(0.0, bounds.x),
                    rand::gen_range(0.0, bounds.y)
                ),
                radius: 5.0,
                absorbed: false
            }
        }
    }

    fn render(&self) {
        self.cell.draw_radius(RED);
    }
}

struct Player {
    cell: Cell,
    speed: f32,
    absorbed_food: i32
}

impl Player {
    fn new() -> Self {
        Self {
            cell: Cell {
                center: Vec2 { x: screen_width() / 2.0, y: screen_height() / 2.0 },
                radius: 10.0,
                absorbed: false
            },
            speed: 80.0,
            absorbed_food: 0
        }
    }

    fn update(&mut self, camera: &mut Camera2D, food: &mut [Food]) {
        let mouse_pos = camera.screen_to_world(mouse_position().into());
        let dir = (mouse_pos - self.cell.center).normalize();

        self.cell.center += dir * self.speed * get_frame_time();
        camera.target = self.cell.center;

        for f in food {
            if let Some(growth_factor) = self.cell.try_absorb(&mut f.cell) {
                self.absorbed_food += 1;
                self.speed -= growth_factor;
            }
        }
    }

    fn render(&self) {
        self.cell.draw_radius(DARKBLUE);
    }
}

struct Creature {
    cell: Cell,
    speed: f32
}

impl Creature {
    fn random(bounds: Vec2) -> Self {
        Self {
            cell: Cell {
                center: vec2(
                    rand::gen_range(0.0, bounds.x),
                    rand::gen_range(0.0, bounds.y)
                ),
                radius: 8.0,
                absorbed: false
            },
            speed: 65.0
        }
    }

    fn update(&mut self, food: &mut [Food], player: &mut Option<Player>, opps: &mut [Creature]) {
        if let Some(p) = player {
            self.speed -= self.cell.try_absorb(&mut p.cell).unwrap_or(0.0);
            p.speed -= p.cell.try_absorb(&mut self.cell).unwrap_or(0.0);
        }
        
        for o in opps {
            self.speed -= self.cell.try_absorb(&mut o.cell).unwrap_or(0.0);
            o.speed -= o.cell.try_absorb(&mut self.cell).unwrap_or(0.0);
        }

        if let Some(nearby_food) = food.iter_mut().min_by_key(|f| {
            self.cell.center.distance_squared(f.cell.center) as i32
        }) {
            let dir = (nearby_food.cell.center - self.cell.center).normalize();

            self.cell.center += dir * self.speed * get_frame_time();
            self.speed -= self.cell.try_absorb(&mut nearby_food.cell).unwrap_or(0.0);
        }
    }

    fn render(&self) {
        self.cell.draw_radius(ORANGE);
    }
}

struct World {
    bounds: Vec2,
    food: Vec<Food>,
    player: Option<Player>,
    creatures: Vec<Creature>
}

impl World {
    fn new(bounds: Vec2) -> Self {
        Self {
            bounds,
            food: (0..(bounds.x / 2.0) as i32).map(|_| Food::random(bounds)).collect(),
            player: Some(Player::new()),
            creatures: (0..99).map(|_| Creature::random(bounds)).collect()
        }
    }
}

#[derive(PartialEq)]
enum GameState {
    Playing,
    Win,
    Lose
}

struct Game {
    state: GameState,
    world: World,
    camera: Camera2D
}

impl Game {
    fn new() -> Self {
        Self {
            state: GameState::Playing,
            world: World::new(vec2(2048.0, 2048.0)),
            camera: Camera2D::from_display_rect(Rect::new(
                0.0, 0.0, screen_width(), -screen_height()
            ))
        }
    }

    fn update(&mut self) {
        if self.state != GameState::Playing {
            return
        }

        if let Some(p) = &mut self.world.player {
            p.update(&mut self.camera, &mut self.world.food);
        } else {
            if let Some(c) = self.world.creatures.iter().max_by(|a, b| {
                a.cell.radius.partial_cmp(&b.cell.radius).unwrap()
            }) {
                self.camera.target = c.cell.center;
            }
        }

        for c_idx in 0..self.world.creatures.len() + 1 {
            let (creatures, opp_creatures) = self.world.creatures.split_at_mut(c_idx);

            if let Some(c) = creatures.last_mut() {
                c.update(&mut self.world.food, &mut self.world.player, opp_creatures);
            }
        }

        self.world.creatures.retain(|c| !c.cell.absorbed);
        self.world.food.retain(|f| !f.cell.absorbed);

        if self.world.player.as_ref().filter(|p| p.cell.absorbed).is_some() {
            self.world.player = None;
        }

        while self.world.food.len() < (self.world.bounds.x / 2.0) as usize  {
            self.world.food.push(Food::random(self.world.bounds));
        }

        if self.world.player.is_some() && self.world.creatures.is_empty() {
            self.state = GameState::Win;
        } else if self.world.player.is_none() && self.world.creatures.len() == 1 {
            self.state = GameState::Lose;
        }
    }

    fn render(&mut self) {
        match self.state {
            GameState::Playing => {
                draw_rectangle(0.0, 0.0, self.world.bounds.x, self.world.bounds.y, WHITE);

                root_ui().label(None, &format!("FPS: {}", get_fps()));
                root_ui().label(None, &format!("Creatures: {}", self.world.creatures.len()));

                for f in &self.world.food {
                    f.render();
                }

                for c in &self.world.creatures {
                    c.render();
                }

                if let Some(p) = &self.world.player {
                    p.render();
                    
                    root_ui().label(None, &format!("Player Radius: {:.2}", p.cell.radius));
                    root_ui().label(None, &format!("Food eaten: {}", p.absorbed_food));
                }
                
                set_camera(&self.camera);

                self.leaderboard(10);
            }
            GameState::Win | GameState::Lose => {
                set_default_camera();
    
                let text = match self.state {
                    GameState::Win => "You Win!",
                    GameState::Lose => "You Lose!",
                    _ => unreachable!()
                };
                let text_width = measure_text(text, None, 100, 1.0).width;
                let screen_center = self.camera.world_to_screen(self.camera.target);
    
                draw_text(
                    text,
                    screen_center.x - text_width / 2.0,
                    screen_center.y,
                    100.0, 
                    if self.state == GameState::Win { GREEN } else { RED }
                );
            }
        }
    }

    fn leaderboard(&self, cnt: usize) {
        let mut leaderboard: Vec<_> = self.world.creatures
            .iter()
            .enumerate()
            .map(|(i, c)| (format!("Creature {}", i + 1), c.cell.radius))
            .collect();
    
        if let Some(player) = &self.world.player {
            leaderboard.push(("Player".to_string(), player.cell.radius));
        }
    
        leaderboard.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        leaderboard.truncate(cnt);

        for (index, (name, radius)) in leaderboard.iter().enumerate() {
            let text = format!("{}. {}: {:.2}", index + 1, name, radius);
            
            root_ui().label(Some(vec2(screen_width() - 160.0, (index * 20) as f32)), &text);
        }        
    }
}

#[macroquad::main("Agar")]
async fn main() {
    let mut game = Game::new();

    loop {
        clear_background(SKYBLUE);

        game.update();
        game.render();

        next_frame().await;
    }
}
