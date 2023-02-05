use super::*;

pub struct EndpointToolConfig {}

impl EditorToolConfig for EndpointToolConfig {
    fn default(assets: &Assets) -> Self {
        Self {}
    }
}

pub struct EndpointTool {
    geng: Geng,
    assets: Rc<Assets>,
    config: EndpointToolConfig,
}

impl EditorTool for EndpointTool {
    type Config = EndpointToolConfig;
    fn new(geng: &Geng, assets: &Rc<Assets>, config: EndpointToolConfig) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            config,
        }
    }
    fn draw(
        &self,
        cursor: &Cursor,
        level: &Level,
        camera: &geng::Camera2d,
        framebuffer: &mut ugli::Framebuffer,
    ) {
    }
    fn handle_event(&mut self, cursor: &Cursor, event: &geng::Event, level: &mut Level) {
        match event {
            geng::Event::MouseDown {
                button: geng::MouseButton::Left,
                ..
            } => {
                level.modify().spawn_point = cursor.world_pos;
            }
            geng::Event::MouseDown {
                button: geng::MouseButton::Right,
                ..
            } => {
                level.modify().finish_point = cursor.world_pos;
            }
            _ => {}
        }
    }

    const NAME: &'static str = "Endpoint";

    fn ui<'a>(&'a mut self, cx: &'a geng::ui::Controller) -> Box<dyn geng::ui::Widget + 'a> {
        Box::new(geng::ui::column![
            "left click changes spawn",
            "right click changes finish",
        ])
    }
}
