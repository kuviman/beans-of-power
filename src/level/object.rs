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
    geng: &Geng,
    path: &std::path::Path,
) -> geng::AssetFuture<HashMap<String, Texture>> {
    let geng = geng.clone();
    let path = path.to_owned();
    async move {
        let json = <String as geng::LoadAsset>::load(&geng, &path.join("_list.json")).await?;
        let list: Vec<String> = serde_json::from_str(&json).unwrap();
        future::join_all(list.into_iter().map(|name| {
            let geng = geng.clone();
            let path = path.clone();
            async move {
                Ok((
                    name.clone(),
                    geng::LoadAsset::load(&geng, &path.join(format!("{}.png", name))).await?,
                ))
            }
        }))
        .await
        .into_iter()
        .collect::<Result<_, anyhow::Error>>()
    }
    .boxed_local()
}
