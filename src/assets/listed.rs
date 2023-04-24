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

impl<T: geng::asset::Load> Listed<T> {
    // TODO remove
    pub async fn load_with_ext(
        manager: &geng::asset::Manager,
        path: &std::path::Path,
        ext: Option<&str>,
    ) -> anyhow::Result<Self> {
        let list: Vec<String> = file::load_detect(path.join("_list.ron")).await?;
        Ok(Self {
            list: futures::future::try_join_all(list.iter().map(|name| {
                manager.load(match ext.or(T::DEFAULT_EXT) {
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
}

impl<T: geng::asset::Load> geng::asset::Load for Listed<T> {
    fn load(manager: &geng::asset::Manager, path: &std::path::Path) -> geng::asset::Future<Self> {
        let manager = manager.clone();
        let path = path.to_owned();
        async move { Self::load_with_ext(&manager, &path, None).await }.boxed_local()
    }

    const DEFAULT_EXT: Option<&'static str> = None;
}
