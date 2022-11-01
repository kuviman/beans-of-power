use super::*;

mod object;
mod surface;
mod tile;

pub use object::*;
pub use surface::*;
pub use tile::*;

pub trait EditorTool: 'static {
    const NAME: &'static str;
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
    fn ui<'a>(&'a mut self, cx: &'a geng::ui::Controller) -> Box<dyn geng::ui::Widget + 'a>;
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
    fn ui<'a>(&'a mut self, cx: &'a geng::ui::Controller) -> Box<dyn geng::ui::Widget + 'a>;
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
    fn ui<'a>(&'a mut self, cx: &'a geng::ui::Controller) -> Box<dyn geng::ui::Widget + 'a> {
        <T as EditorTool>::ui(self, cx)
    }
}

pub trait ToolConstructor {
    fn name(&self) -> &str;
    fn create(&self) -> Box<dyn DynEditorTool>;
}

pub fn tool_constructor<T: EditorTool>(
    geng: &Geng,
    assets: &Rc<Assets>,
) -> Box<dyn ToolConstructor> {
    struct Thing<T: EditorTool> {
        geng: Geng,
        assets: Rc<Assets>,
        phantom_data: PhantomData<T>,
    }
    impl<T: EditorTool> ToolConstructor for Thing<T> {
        fn name(&self) -> &str {
            T::NAME
        }
        fn create(&self) -> Box<dyn DynEditorTool> {
            Box::new(T::new(
                &self.geng,
                &self.assets,
                T::Config::default(&self.assets),
            ))
        }
    }
    Box::new(Thing::<T> {
        geng: geng.clone(),
        assets: assets.clone(),
        phantom_data: PhantomData,
    })
}
