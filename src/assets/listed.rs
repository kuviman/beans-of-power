use super::*;

pub struct Listed<T> {
    list: Vec<T>,
    name_to_index: HashMap<String, usize>,
}

impl<T> Listed<T> {
    pub fn index(&self, name: &str) -> Option<usize> {
        self.name_to_index.get(name).copied()
    }
    pub fn get(&self, name: &str) -> Option<&T> {
        self.index(name).map(|index| &self.list[index])
    }
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.name_to_index.keys().map(|s| s.as_str())
    }
}

impl<T> Index<&str> for Listed<T> {
    type Output = T;
    fn index(&self, index: &str) -> &T {
        self.get(index).unwrap()
    }
}

impl<T: geng::LoadAsset> geng::LoadAsset for Listed<T> {
    fn load(geng: &Geng, path: &std::path::Path) -> geng::AssetFuture<Self> {
        let geng = geng.clone();
        let path = path.to_owned();
        async move {
            let list: Vec<String> = file::load_detect(path.join("_list.ron")).await?;
            Ok(Self {
                list: futures::future::try_join_all(list.iter().map(|name| {
                    geng.load_asset(match T::DEFAULT_EXT {
                        Some(ext) => path.join(format!("{name}.{ext}")),
                        None => path.join(name),
                    })
                }))
                .await?,
                name_to_index: list
                    .into_iter()
                    .enumerate()
                    .map(|(index, name)| (name, index))
                    .collect(),
            })
        }
        .boxed_local()
    }

    const DEFAULT_EXT: Option<&'static str> = None;
}
