use super::*;

mod surface;
mod tile;

pub use surface::*;
pub use tile::*;

pub trait EditorTool: 'static {
    type Config: EditorToolConfig;
    fn new(geng: &Geng, assets: &Rc<Assets>, config: Self::Config) -> Self;
    fn draw(
        &self,
        cursor: &Cursor,
        level: &Level,
        camera: &geng::Camera2d,
        framebuffer: &mut ugli::Framebuffer,
    );
    fn handle_event(&mut self, cursor: &Cursor, event: &geng::Event, level: &mut Level);
}

pub trait EditorToolConfig {
    fn default(assets: &Assets) -> Self;
}

pub trait DynEditorTool {
    fn draw(
        &self,
        cursor: &Cursor,
        level: &Level,
        camera: &geng::Camera2d,
        framebuffer: &mut ugli::Framebuffer,
    );
    fn handle_event(&mut self, cursor: &Cursor, event: &geng::Event, level: &mut Level);
}

impl<T: EditorTool> DynEditorTool for T {
    fn draw(
        &self,
        cursor: &Cursor,
        level: &Level,
        camera: &geng::Camera2d,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        <T as EditorTool>::draw(self, cursor, level, camera, framebuffer)
    }
    fn handle_event(&mut self, cursor: &Cursor, event: &geng::Event, level: &mut Level) {
        <T as EditorTool>::handle_event(self, cursor, event, level)
    }
}
