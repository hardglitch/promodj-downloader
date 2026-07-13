use crate::logic::dsl::{Command, Data};
use crate::logic::tools;
use crate::logic::tools::clear_filename;
use futures_util::StreamExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::Receiver;
use tokio::sync::RwLock;

pub async fn download_files(
    links: &[String],
    save_to: &Path,
    client: reqwest::Client,
    overwrite_files: bool,
    rx: Arc<RwLock<Receiver<Data>>>,
)
    -> anyhow::Result<()>
{
    for link in links.iter() {
        let mut file = DlFile::new(link, save_to)?;
        file.download(client.clone(), overwrite_files, rx.clone()).await?;
    }
    Ok(())
}

#[derive(Debug)]
pub(crate) struct DlFile<'a> {
    link: &'a str,
    name: String,
    path: PathBuf,
}
impl<'a> DlFile<'a> {
    pub(crate) fn new(link: &'a str, save_to: &Path) -> anyhow::Result<Self> {
        let name = match link.rsplit('/').next() {
            Some(n) => n,
            None => return Err(anyhow::anyhow!("Bad the file name"))
        };
        let name = clear_filename(name);
        let path = save_to.join(&name);
        Ok(Self { link, name, path })
    }

    pub(crate) async fn download(
        &mut self,
        client: reqwest::Client,
        overwrite: bool,
        rx: Arc<RwLock<Receiver<Data>>>
    )
        -> anyhow::Result<()>
    {
        if self.path.exists() && !overwrite {
            let new_filename = match tools::new_filename(&self.name) {
                Some(n) => n,
                None => return Err(anyhow::anyhow!("Bad the new file name"))
            };
            let new_path = self.path.join(&new_filename);
            self.name = new_filename;
            self.path = new_path;
        }

        let response =
            client
                .get(self.link)
                .timeout(Duration::from_secs(u64::MAX))
                // .header("Connection", "keep-alive")
                .send()
                .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Bad status = {}", response.status()));
        }

        let mut file = tokio::fs::File::create(&self.path).await?;
        let mut stream = response.bytes_stream();
        while let Some(Ok(chunk)) = stream.next().await {
            // if let Some(data) = rx.write().await.recv().await {
            //     match data.command() {
            //         Command::Stop => break,
            //         Command::Pause => {
            //             loop {
            //                 if let Some(data) = rx.write().await.recv().await &&
            //                     matches!(data.command(), Command::Start)
            //                 { break }
            //             }
            //         }
            //         _ => {}
            //     }
            // }

            file.write_all(&chunk).await?;
        }
        // if let Some(data) = rx.write().await.recv().await &&
        //     matches!(data.command(), Command::Stop)
        // {
        //     tokio::fs::remove_file(&self.path).await?;
        // }
        Ok(())
    }
}