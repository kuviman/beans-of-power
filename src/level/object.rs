use super::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Object {
    pub type_name: String,
    pub pos: vec2<f32>,
}

impl Object {
    // TODO: this should go into config I guess
    pub fn fart_type(&self) -> Option<&str> {
        Some(match self.type_name.as_str() {
            "unicorn" => "rainbow",
            "hot-pepper" => "fire",
            "guitar" => "melody",
            _ => return None,
        })
    }
}

pub fn load_objects_assets(
    manager: &geng::asset::Manager,
    path: &std::path::Path,
) -> geng::asset::Future<HashMap<String, Texture>> {
    let manager = manager.clone();
    let path = path.to_owned();
    async move {
        let list: Vec<String> = file::load_detect(path.join("_list.json")).await?;
        future::join_all(list.into_iter().map(|name| {
            let manager = manager.clone();
            let path = path.clone();
            async move {
                Ok((
                    name.clone(),
                    manager.load(path.join(format!("{}.png", name))).await?,
                ))
            }
        }))
        .await
        .into_iter()
        .collect::<Result<_, anyhow::Error>>()
    }
    .boxed_local()
}
