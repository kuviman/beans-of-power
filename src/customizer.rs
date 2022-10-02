use super::*;

#[derive(Clone)]
enum UiMessage {
    Play,
}

pub struct Customizer {
    geng: Geng,
    ui_controller: ui::Controller,
    buttons: Vec<ui::Button<UiMessage>>,
    assets: Rc<Assets>,
    f: Option<Box<dyn FnOnce(Guy) -> Game>>,
    transition: Option<geng::Transition>,
    guy: Guy,
}

impl Customizer {
    pub fn new(geng: &Geng, assets: &Rc<Assets>, f: impl FnOnce(Guy) -> Game + 'static) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            f: Some(Box::new(f)),
            ui_controller: ui::Controller::new(geng, assets),
            buttons: vec![ui::Button::new(
                "PLAY",
                vec2(0.0, -3.0),
                1.0,
                0.5,
                UiMessage::Play,
            )],
            transition: None,
            guy: Guy::new(-1, vec2(0.0, 0.0)),
        }
    }
}

impl geng::State for Customizer {
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        let camera = geng::Camera2d {
            center: Vec2::ZERO,
            rotation: 0.0,
            fov: 10.0,
        };
        self.ui_controller
            .draw(framebuffer, &camera, self.buttons.clone());
        self.assets.font.draw(
            framebuffer,
            &camera,
            &self.guy.name,
            vec2(0.0, 3.0),
            geng::TextAlign::CENTER,
            1.0,
            Rgba::new(0.5, 0.5, 1.0, 1.0),
        );
    }
    fn handle_event(&mut self, event: geng::Event) {
        for msg in self
            .ui_controller
            .handle_event(&event, self.buttons.clone())
        {
            match msg {
                UiMessage::Play => {
                    self.transition =
                        Some(geng::Transition::Switch(Box::new(self.f.take().unwrap()(
                            self.guy.clone(),
                        ))))
                }
            }
        }
        match event {
            geng::Event::KeyDown { key } => {
                let s = format!("{:?}", key);
                if s.len() == 1 && self.guy.name.len() < 15 {
                    self.guy.name.push_str(&s);
                }
                if key == geng::Key::Backspace {
                    self.guy.name.pop();
                }
            }
            _ => {}
        }
    }
    fn transition(&mut self) -> Option<geng::Transition> {
        self.transition.take()
    }
}
