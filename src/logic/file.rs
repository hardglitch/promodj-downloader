use crate::logic::{tools, Command};
use crate::logic::tools::clear_filename;
use futures_util::StreamExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::RwLock;

pub async fn download_files(
    links: &[String],
    save_to: &Path,
    client: reqwest::Client,
    overwrite_files: bool,
    common_tx: Arc<RwLock<UnboundedSender<Command>>>,
    control_rx: Arc<RwLock<UnboundedReceiver<Command>>>,
)
    -> anyhow::Result<()>
{
    let total_links = links.len();
    for (link_number, link) in links.iter().enumerate() {
        let mut file = DlFile::new(link, save_to)?;
        file.download(
            client.clone(),
            overwrite_files,
            common_tx.clone(),
            control_rx.clone(),
            link_number + 1,
            total_links
        ).await?;
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
        common_tx: Arc<RwLock<UnboundedSender<Command>>>,
        control_rx: Arc<RwLock<UnboundedReceiver<Command>>>,
        file_number: usize,
        total_files: usize,
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
        let file_length = response.content_length();
        let mut stream = response.bytes_stream();
        let mut total_downloaded = 0;
        let mut p2 = 0.;

        while let Some(Ok(chunk)) = stream.next().await {
            file.write_all(&chunk).await?;
            total_downloaded += chunk.len();

            // Progress info
            let progress =
                if let Some(file_length) = file_length && file_length > 0 {
                    if file_number == 1 {
                        total_downloaded as f32 / file_length as f32
                    }
                    else {
                        (file_number.saturating_sub(1) as f32 / total_files as f32) * (1. + (total_downloaded as f32 / file_length as f32))
                    }
                }
                else { 0. };

            let p1 = (progress * 100.0).round();
            if p1 > p2 || progress == 0. || progress == 100. {
                if let Ok(tx) = common_tx.try_read() {
                    tx.send(Command::Progress(progress))?;
                }
                p2 = p1;
            }

            // Control
            // if let Ok(mut rx) = control_rx.try_write() &&
            //    let Ok(data) = rx.recv()
            // {
            //     match data.command() {
            //         Command::Stop => break,
            //         Command::Pause => {
            //             loop {
            //                 if let Some(data) = rx1.write().await.recv().await &&
            //                     matches!(data.command(), Command::Start)
            //                 { break }
            //             }
            //         }
            //         _ => {}
            //     }
            // }
        }

        // if let Some(data) = rx1.write().await.recv() &&
        //    matches!(data.command(), Command::Stop)
        // {
        //     tokio::fs::remove_file(&self.path).await?;
        // }

        Ok(())
    }
}