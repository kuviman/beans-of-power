use super::*;

pub use resvg::usvg::{Node, Tree};

pub async fn load(path: impl AsRef<std::path::Path>) -> anyhow::Result<Tree> {
    let svg_data: Vec<u8> = file::load_bytes(path).await?;
    let tree = resvg::usvg::Tree::from_data(&svg_data, &resvg::usvg::Options::default())?;
    Ok(tree)
}

pub fn render(geng: &Geng, tree: &Tree, node: Option<&Node>) -> ugli::Texture {
    let size = tree.size.to_screen_size();
    let mut pixmap = resvg::tiny_skia::Pixmap::new(size.width(), size.height())
        .expect("Failed to create pixmap");
    match node {
        None => {
            resvg::render(
                tree,
                resvg::usvg::FitTo::Original,
                resvg::tiny_skia::Transform::identity(),
                pixmap.as_mut(),
            );
        }
        Some(node) => {
            // TODO: NO IDEA HOW TO USE resvg::render_to_node PROPERLY
            let mut tree = tree.clone();
            tree.root = node.clone(); // TODO this is wrong?
            resvg::render(
                &tree,
                resvg::usvg::FitTo::Original,
                resvg::tiny_skia::Transform::identity(),
                pixmap.as_mut(),
            );
        }
    };
    let mut image_data = pixmap.take();
    for color in image_data.chunks_mut(4) {
        let color: &mut [u8; 4] = color.try_into().unwrap();
        let premultiplied_color: resvg::tiny_skia::PremultipliedColorU8 =
            *bytemuck::cast_ref(color);
        let rgba = premultiplied_color.demultiply();
        // ColorU8 is not Pod WUT: *bytemuck::cast_mut(color) = rgba;
        color[0] = rgba.red();
        color[1] = rgba.green();
        color[2] = rgba.blue();
        color[3] = rgba.alpha();
    }
    let image = image::RgbaImage::from_vec(size.width(), size.height(), image_data).unwrap();
    ugli::Texture::from_image_image(geng.ugli(), image)
}
