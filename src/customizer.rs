use super::*;

#[derive(Clone)]
pub enum UiMessage {
    Play,
    RandomizeSkin,
}

impl Game {
    pub fn draw_customizer(&mut self, framebuffer: &mut ugli::Framebuffer) {
        if !self.show_customizer {
            return;
        }
        let camera = geng::Camera2d {
            center: Vec2::ZERO,
            rotation: 0.0,
            fov: 10.0,
        };
        self.ui_controller
            .draw(framebuffer, &camera, self.buttons.clone());
        if self.customization.name.is_empty() {
            self.assets.font.draw(
                framebuffer,
                &camera,
                "type your name",
                vec2(0.0, 3.0),
                geng::TextAlign::CENTER,
                1.0,
                Rgba::new(0.5, 0.5, 1.0, 0.5),
            );
            self.assets.font.draw(
                framebuffer,
                &camera,
                "yes just type it",
                vec2(0.0, 2.0),
                geng::TextAlign::CENTER,
                1.0,
                Rgba::new(0.5, 0.5, 1.0, 0.5),
            );
        } else {
            self.assets.font.draw(
                framebuffer,
                &camera,
                &self.customization.name,
                vec2(0.0, 3.0),
                geng::TextAlign::CENTER,
                1.0,
                Rgba::new(0.5, 0.5, 1.0, 1.0),
            );
        }
    }

    pub fn handle_customizer_event(&mut self, event: &geng::Event) {
        if !self.show_customizer {
            return;
        }
        for msg in self.ui_controller.handle_event(event, self.buttons.clone()) {
            match msg {
                UiMessage::Play => {
                    self.show_customizer = false;
                }
                UiMessage::RandomizeSkin => {
                    self.customization.colors = Guy::new(-1, Vec2::ZERO, true).colors;
                }
            }
        }
        match event {
            geng::Event::KeyDown { key } => {
                let s = format!("{:?}", key);
                if s.len() == 1 && self.customization.name.len() < 15 {
                    self.customization.name.push_str(&s);
                }
                if *key == geng::Key::Backspace {
                    self.customization.name.pop();
                }
            }
            _ => {}
        }
    }
}
