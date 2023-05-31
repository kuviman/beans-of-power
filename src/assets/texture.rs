use super::*;

#[derive(Deref, DerefMut)]
pub struct Texture(#[deref] ugli::Texture);

impl From<ugli::Texture> for Texture {
    fn from(texture: ugli::Texture) -> Self {
        Self(texture)
    }
}

impl std::borrow::Borrow<ugli::Texture> for Texture {
    fn borrow(&self) -> &ugli::Texture {
        &self.0
    }
}
impl std::borrow::Borrow<ugli::Texture> for &'_ Texture {
    fn borrow(&self) -> &ugli::Texture {
        &self.0
    }
}

impl geng::asset::Load for Texture {
    fn load(manager: &geng::asset::Manager, path: &std::path::Path) -> geng::asset::Future<Self> {
        if path.extension() == Some("svg".as_ref()) {
            let manager = manager.clone();
            let path = path.to_owned();
            async move {
                let svg = svg::load(path).await?;
                Ok(Texture(svg::render(manager.ugli(), &svg.tree, None)))
            }
            .boxed_local()
        } else {
            let texture = <ugli::Texture as geng::asset::Load>::load(manager, path);
            async move {
                let mut texture = texture.await?;
                texture.set_filter(ugli::Filter::Nearest);
                Ok(Texture(texture))
            }
            .boxed_local()
        }
    }

    const DEFAULT_EXT: Option<&'static str> = Some("png");
}
