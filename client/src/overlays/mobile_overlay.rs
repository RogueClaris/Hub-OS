use crate::bindable::SpriteColorMode;
use crate::render::{Animator, Camera, SpriteColorQueue};
use crate::resources::{AssetManager, Globals, ResourcePaths, RESOLUTION_F};
use framework::prelude::*;
use std::ops::Range;

const PRESSED_COLOR: Color = Color::new(1.0, 1.0, 1.0, 0.9);
const RELEASED_COLOR: Color = Color::new(1.0, 1.0, 1.0, 0.3);

const BASE: f32 = 64.0 * 3.0;

const LEFT_INPUT_RANGE: Range<usize> = 0..2;
const RIGHT_INPUT_RANGE: Range<usize> = 2..8;

const BUTTON_ORDER: [Button; 8] = [
    Button::LeftTrigger,
    Button::Select,
    Button::RightTrigger,
    Button::Start,
    Button::A,
    Button::B,
    Button::X,
    Button::Y,
];

pub struct MobileOverlay {
    camera: Camera,
    rectangle: FlatModel,
    button_sprites: Vec<(Button, Rect, Sprite)>,
    dpad_sprite: (Rect, Sprite),
    dpad_dead_zone: Rect,
}

impl MobileOverlay {
    pub fn new(game_io: &mut GameIO) -> Self {
        let globals = game_io.resource::<Globals>().unwrap();
        let assets = &globals.assets;

        let sprite = assets.new_sprite(game_io, ResourcePaths::INPUT_OVERLAY);
        let mut animator = Animator::load_new(assets, ResourcePaths::INPUT_OVERLAY_ANIMATION);

        let button_sprites = BUTTON_ORDER
            .into_iter()
            .map(|button| {
                let button_name: &'static str = button.into();
                let mut sprite = sprite.clone();

                animator.set_state(button_name);
                animator.apply(&mut sprite);

                (button, sprite.bounds(), sprite)
            })
            .collect::<Vec<_>>();

        let mut dpad_sprite = sprite;
        animator.set_state("DPAD");
        animator.apply(&mut dpad_sprite);

        let dpad_dead_zone = Rect::from_corners(
            animator.point_or_zero("DEAD_ZONE_START"),
            animator.point_or_zero("DEAD_ZONE_END"),
        );

        Self {
            camera: Camera::new(game_io),
            rectangle: FlatModel::new_square_model(game_io),
            button_sprites,
            dpad_sprite: (dpad_sprite.bounds(), dpad_sprite),
            dpad_dead_zone,
        }
    }

    fn unnormalize(resolution: Vec2, position: Vec2) -> Vec2 {
        (position * Vec2::new(0.5, -0.5) + 0.5) * resolution
    }

    fn touch_positions(game_io: &GameIO) -> Vec<Vec2> {
        let window = game_io.window();
        let scale = window.render_scale();
        let render_offset = window.render_offset();
        let view_size = window.resolution().as_vec2() * scale;

        let touch_iter = game_io.input().touches().iter();

        touch_iter
            .map(|touch| Self::unnormalize(view_size, touch.position) + render_offset)
            .collect()
    }
}

impl GameOverlay for MobileOverlay {
    fn pre_update(&mut self, game_io: &mut GameIO) {
        // update camera
        let window_size = game_io.window().size().as_vec2();
        self.camera.set_scale(RESOLUTION_F / window_size);
        self.camera.snap(window_size * 0.5);

        // update buttons
        let button_scale = window_size.y / RESOLUTION_F.y;

        // update dpad
        {
            let (original_bounds, sprite) = &mut self.dpad_sprite;

            let mut bounds = *original_bounds * button_scale;
            bounds.y += window_size.y;
            sprite.set_bounds(bounds);
        }

        // update left inputs
        for i in LEFT_INPUT_RANGE {
            let (_, original_bounds, sprite) = &mut self.button_sprites[i];

            let mut bounds = *original_bounds * button_scale;
            bounds.y += window_size.y;
            sprite.set_bounds(bounds);
        }

        // update right inputs
        for i in RIGHT_INPUT_RANGE {
            let (_, original_bounds, sprite) = &mut self.button_sprites[i];

            let mut bounds = *original_bounds * button_scale;
            bounds.x += window_size.x;
            bounds.y += window_size.y;
            sprite.set_bounds(bounds);
        }

        // update input using simple buttons
        let touches = Self::touch_positions(game_io);
        let globals = game_io.resource_mut::<Globals>().unwrap();

        globals.emulated_input.flush();

        for (button, _, sprite) in &self.button_sprites {
            let bounds = sprite.bounds();
            let pressed = touches.iter().any(|&position| bounds.contains(position));

            if pressed {
                globals.emulated_input.emulate_button(*button);
            }
        }

        // update input with dpad
        let dpad_bounds = self.dpad_sprite.1.bounds();
        let dead_zone = self.dpad_dead_zone * button_scale;

        for mut position in touches {
            if !dpad_bounds.contains(position) {
                continue;
            }

            position -= dpad_bounds.top_left();

            if dead_zone.contains(position) {
                continue;
            }

            if !dead_zone.horizontal_range().contains(&position.x) {
                if position.x - dpad_bounds.width * 0.5 > 0.0 {
                    globals.emulated_input.emulate_button(Button::DPadRight)
                } else {
                    globals.emulated_input.emulate_button(Button::DPadLeft)
                }
            }

            if !dead_zone.vertical_range().contains(&position.y) {
                if position.y - dpad_bounds.height * 0.5 > 0.0 {
                    globals.emulated_input.emulate_button(Button::DPadDown)
                } else {
                    globals.emulated_input.emulate_button(Button::DPadUp)
                }
            }
        }
    }

    fn draw(&mut self, game_io: &mut GameIO, render_pass: &mut RenderPass) {
        let mut queue = SpriteColorQueue::new(game_io, &self.camera, SpriteColorMode::Multiply);

        let touches = Self::touch_positions(game_io);

        for (_, _, sprite) in &mut self.button_sprites {
            let bounds = sprite.bounds();
            let pressed = touches.iter().any(|&position| bounds.contains(position));

            let color = if pressed {
                PRESSED_COLOR
            } else {
                RELEASED_COLOR
            };

            sprite.set_color(color);
            queue.draw_sprite(sprite);
        }

        self.dpad_sprite.1.set_color(RELEASED_COLOR);
        queue.draw_sprite(&self.dpad_sprite.1);

        #[cfg(debug_assertions)]
        {
            use crate::render::ui::{FontName, TextStyle};

            let window = game_io.window();
            let scale = window.render_scale();
            let render_offset = window.render_offset();
            let view_size = window.resolution().as_vec2() * scale;

            let mut text_style = TextStyle::new(game_io, FontName::ThinSmall);
            text_style.shadow_color = Color::BLACK;
            text_style.scale *= scale;

            for touch in game_io.input().touches() {
                let text = match touch.phase {
                    TouchPhase::Start => "START",
                    TouchPhase::Moving => "MOVING",
                    TouchPhase::End => "END",
                    TouchPhase::Cancelled => "CANCELLED",
                };

                let mut position = Self::unnormalize(view_size, touch.position) + render_offset;
                position -= text_style.measure(text).size * 0.5;
                text_style.bounds.set_position(position);

                text_style.draw(game_io, &mut queue, text);
            }
        }

        render_pass.consume_queue(queue);
    }
}
