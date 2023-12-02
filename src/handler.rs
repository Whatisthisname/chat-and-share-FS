use std::{env::current_dir, io, path::PathBuf};

use crate::{
    req::Request,
    resp::{Response, Status},
};

#[derive(Debug, Clone)]
pub struct StaticFileHandler {
    root: PathBuf,
}

impl StaticFileHandler {
    pub fn in_current_dir() -> io::Result<StaticFileHandler> {
        current_dir().map(StaticFileHandler::with_root)
    }

    pub fn with_root(root: PathBuf) -> StaticFileHandler {
        StaticFileHandler { root }
    }

    pub async fn handle(&self, request: Request) -> anyhow::Result<Response> {
        let path = self.root.join(request.path.strip_prefix('/').unwrap());

        if path.is_dir() {
            Ok(Response::from_index(Status::Ok, path).await)
        } else if path.is_file() {
            let file = tokio::fs::File::open(&path).await?;
            Response::from_file(&path, file).await
        } else {
            //if !path.is_file() {
            Ok(Response::from_html(
                Status::NotFound,
                include_str!("../static/404.html"),
            ))
        }
    }
}
