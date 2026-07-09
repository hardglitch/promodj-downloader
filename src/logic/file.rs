use crate::logic::tools;
use crate::logic::tools::clear_filename;
use futures_util::StreamExt;
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;
use tokio::sync::watch::Receiver;
use crate::logic::dsl::Command;

pub struct DlFile<'a> {
    link: &'a str,
    name: String,
    path: PathBuf,
}
impl<'a> DlFile<'a> {
    pub fn new(link: &'a str, save_to: &Path) -> anyhow::Result<Self> {
        let name = match link.rsplit('/').next() {
            Some(n) => n,
            None => return Err(anyhow::anyhow!("Bad the file name"))
        };
        let name = clear_filename(name);
        let path = save_to.join(&name);
        Ok(Self { link, name, path })
    }

    pub async fn download(&mut self, overwrite: bool, control_rx: Receiver<Command>) -> anyhow::Result<()> {
        if self.path.exists() && !overwrite {
            let new_filename = match tools::new_filename(&self.name) {
                Some(n) => n,
                None => return Err(anyhow::anyhow!("Bad the new file name"))
            };
            let new_path = self.path.join(&new_filename);
            self.name = new_filename;
            self.path = new_path;
        }

        let client = reqwest::Client::new();
        let response = client.get(self.link).send().await?;
        if response.status() != 200 {
            return Err(anyhow::anyhow!("Bad status = {}", response.status()));
        }

        let mut file = tokio::fs::File::create(&self.path).await?;
        let mut stream = response.bytes_stream();
        while let Some(Ok(chunk)) = stream.next().await {
            match *control_rx.borrow() {
                Command::Stop => break,
                Command::Pause => {
                    loop {
                        if matches!(*control_rx.borrow(), Command::Start) { break }
                    }
                }
                _ => {}
            }

            file.write_all(&chunk).await?;
        }
        if matches!(*control_rx.borrow(), Command::Stop) {
            tokio::fs::remove_file(&self.path).await?;
        }
        Ok(())
    }
}
