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
            center: vec2::ZERO,
            rotation: 0.0,
            fov: 10.0,
        };
        self.ui_controller
            .draw(framebuffer, &camera, self.buttons.clone());
        if self.customization.name.is_empty() {
            self.assets.get().font.draw(
                framebuffer,
                &camera,
                "type your name",
                vec2::splat(geng::TextAlign::CENTER),
                mat3::translate(vec2(0.0, 3.0)),
                Rgba::new(0.5, 0.5, 1.0, 0.5),
            );
            self.assets.get().font.draw(
                framebuffer,
                &camera,
                "yes just type it",
                vec2::splat(geng::TextAlign::CENTER),
                mat3::translate(vec2(0.0, 2.0)),
                Rgba::new(0.5, 0.5, 1.0, 0.5),
            );
        } else {
            self.assets.get().font.draw(
                framebuffer,
                &camera,
                &self.customization.name,
                vec2::splat(geng::TextAlign::CENTER),
                mat3::translate(vec2(0.0, 3.0)),
                Rgba::new(0.5, 0.5, 1.0, 1.0),
            );
        }
    }

    pub fn handle_customizer_event(&mut self, event: &geng::Event) {
        if !self.show_customizer {
            if matches!(
                event,
                geng::Event::KeyDown {
                    key: geng::Key::Enter,
                } | geng::Event::Gamepad(gilrs::Event {
                    event: gilrs::EventType::ButtonPressed(gilrs::Button::Start, ..),
                    ..
                })
            ) {
                self.show_customizer = true;
            }
            return;
        }
        let msgs = self
            .ui_controller
            .handle_event(event, self.buttons.clone())
            .into_iter();
        let msgs = msgs.chain(
            matches!(
                event,
                geng::Event::KeyDown {
                    key: geng::Key::Enter,
                } | geng::Event::Gamepad(gilrs::Event {
                    event: gilrs::EventType::ButtonPressed(gilrs::Button::Start, ..),
                    ..
                })
            )
            .then_some(UiMessage::Play),
        );
        for msg in msgs {
            match msg {
                UiMessage::Play => {
                    self.show_customizer = false;
                    preferences::save("customization", &self.customization);
                }
                UiMessage::RandomizeSkin => {
                    self.customization.colors = GuyColors::random();
                }
            }
        }
        if let geng::Event::KeyDown { key } = event {
            let s = format!("{key:?}");
            let c = if s.len() == 1 {
                Some(s.as_str())
            } else {
                s.strip_prefix("Num")
            };
            if let Some(c) = c {
                if self.customization.name.len() < 15 {
                    self.customization.name.push_str(c);
                }
            }
            if *key == geng::Key::Backspace {
                self.customization.name.pop();
            }
        }
    }
}
