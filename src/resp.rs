use std::{
    collections::HashMap,
    fmt::{Debug, Display, Formatter},
    io::Cursor,
    path::{Path, PathBuf}, fs::DirEntry,
};

use maplit::hashmap;
use tokio::{
    fs::{File},
    io::{AsyncRead, AsyncWrite, AsyncWriteExt},
};

pub struct Response {
    pub status: Status,
    pub headers: HashMap<String, String>,
    pub data: Box<dyn AsyncRead + Unpin + Send>,
}

impl Response {
    pub fn status_and_headers(&self) -> String {
        let headers = self
            .headers
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<_>>()
            .join("\r\n");

        format!("HTTP/1.1 {}\r\n{headers}\r\n\r\n", self.status)
    }

    pub async fn write<O: AsyncWrite + Unpin>(mut self, stream: &mut O) -> anyhow::Result<()> {
        let bytes = self.status_and_headers().into_bytes();

        stream.write_all(&bytes).await?;

        tokio::io::copy(&mut self.data, stream).await?;

        Ok(())
    }

    pub fn from_html(status: Status, data: impl ToString) -> Self {
        let bytes = data.to_string().into_bytes();

        let headers = hashmap! {
            "Content-Type".to_string() => "text/html".to_string(),
            "Content-Length".to_string() => bytes.len().to_string(),
        };

        Self {
            status,
            headers,
            data: Box::new(Cursor::new(bytes)),
        }
    }
    
    pub async fn from_index(status: Status, root_path: PathBuf) -> Self {
        // let bytes = data.to_string().into_bytes();
        let template = include_str!("../static/index.html").to_string();

        // read local directory at pat
        // for each file, add to list

        let mut filenames : Vec<DirEntry> = std::fs::read_dir(&root_path).unwrap().filter(|r| r.is_ok()).map(|r| r.unwrap()).collect();

        let mut list_items = String::new();
        let root_str = root_path.to_str().unwrap().to_string();
        let base_string =root_str.strip_suffix("/").unwrap_or(&root_str);

        // sort filenames by (is_dir, name)
        filenames.sort_by(|a, b| {
            let a_is_dir = a.file_type().unwrap().is_dir();
            let b_is_dir = b.file_type().unwrap().is_dir();
            if a_is_dir && !b_is_dir {
                std::cmp::Ordering::Less
            } else if !a_is_dir && b_is_dir {
                std::cmp::Ordering::Greater
            } else {
                a.file_name().cmp(&b.file_name())
            }
        });

        for filename in filenames {
            let name = filename.file_name().to_str().unwrap().to_string();
            let path = filename.path().as_os_str().to_str().unwrap().to_string();

            let item = format!("<li><a href=\"{0}/{1}\">{1}</a></li>", base_string, name);
            list_items.push_str(item.as_str());
        }

       
        // now, we combine the first part with the list items and the last part

        let html = template.replace(r"{{LIST_ITEMS}}", list_items.as_str());

        let bytes = html.into_bytes();

        let headers = hashmap! {
            "Content-Type".to_string() => "text/html".to_string(),
            "Content-Length".to_string() => bytes.len().to_string(),
        };

        Self {
            status,
            headers,
            data: Box::new(Cursor::new(bytes)),
        }
    }
    pub async fn from_file(path: &Path, file: File) -> anyhow::Result<Response> {
        let headers = hashmap! {
            "Content-Length".to_string() => file.metadata().await?.len().to_string(),
            "Content-Type".to_string() => mime_type(path).to_string(),
        };

        Ok(Response {
            headers,
            status: Status::Ok,
            data: Box::new(file),
        })
    }



}

fn mime_type(path: &Path) -> &str {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("html") => "text/html",
        Some("css") => "text/css",
        Some("js") => "text/javascript",
        Some("png") => "image/png",
        Some("jpg") => "image/jpeg",
        Some("gif") => "image/gif",
        _ => "application/octet-stream",
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Status {
    NotFound,
    Ok,
}

impl Display for Status {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::NotFound => write!(f, "404 Not Found"),
            Status::Ok => write!(f, "200 OK"),
        }
    }
}