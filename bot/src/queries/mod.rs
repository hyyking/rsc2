use moka::future::Cache;
use std::{
    cell::LazyCell,
    path::{Path, PathBuf},
    time::Duration,
};

const CURRENT_DIR: LazyCell<PathBuf> =
    LazyCell::new(|| Path::new(file!()).parent().unwrap().to_path_buf());

const QUERY_CACHE: LazyCell<Cache<String, String>> = LazyCell::new(|| {
    Cache::builder()
        .max_capacity(64)
        .time_to_idle(Duration::from_secs(15))
        .build()
});

pub async fn get(name: impl ToString) -> Option<String> {
    let name = name.to_string();
    QUERY_CACHE
        .optionally_get_with(name.clone(), async move {
            let path = CURRENT_DIR.join(format!("{}.surql", name));

            if !path.exists() {
                return None;
            }

            let content = tokio::fs::read_to_string(&path).await.ok()?;
            Some(content)
        })
        .await
}
