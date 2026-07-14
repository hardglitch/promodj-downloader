use crate::logic::tools::clear_filename;
use crate::logic::{tools, Command};
use futures_util::StreamExt;
use percent_encoding::percent_decode;
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
    -> anyhow::Result<Option<Command>>
{
    let total_links = links.len();
    for (link_number, link) in links.iter().enumerate() {
        let mut file = DlFile::new(link, save_to)?;
        let res =
            file.download(
                client.clone(),
                overwrite_files,
                common_tx.clone(),
                control_rx.clone(),
                link_number + 1,
                total_links
            ).await?;

        if matches!(res, Some(Command::Stop)) { return Ok(res) }
    }
    Ok(None)
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
            Some(n) => percent_decode(n.as_bytes()).decode_utf8()?,
            None => return Err(anyhow::anyhow!("Bad the file name"))
        };
        let name = clear_filename(&name);
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
        -> anyhow::Result<Option<Command>>
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
        let mut downloaded = 0;
        let mut p2 = 0.;
        let step = 1. / total_files as f32;
        let is_canceled = &mut false;

        while let Some(Ok(chunk)) = stream.next().await {
            file.write_all(&chunk).await?;
            downloaded += chunk.len();

            // Progress info
            let progress =
                if let Some(file_length) = file_length && file_length > 0 && total_files > 0 {
                    let shift = file_number.saturating_sub(1) as f32 / total_files as f32;
                    let file_progress = downloaded as f32 / file_length as f32;

                    shift + step * file_progress
                } else { return Err(anyhow::anyhow!("Bad the progress value")) };

            let p1 = (progress * 100.0).round();
            if p1 > p2 || progress == 0. {
                if let Ok(tx) = common_tx.try_read() {
                    tx.send(Command::Progress(progress, file_number, total_files))?;
                }
                p2 = p1;
            }

            // Control
            if let Ok(mut rx) = control_rx.try_write() &&
               let Ok(cmd) = rx.try_recv()
            {
                match cmd {
                    Command::Stop => {
                        *is_canceled = true;
                        break
                    },
                    Command::Pause => {
                        loop {
                            if let Some(cmd) = control_rx.write().await.recv().await &&
                                matches!(cmd, Command::Start)
                            { break }
                        }
                    }
                    _ => {}
                }
            }
        }

        if *is_canceled {
            tokio::fs::remove_file(&self.path).await?;
            return Ok(Some(Command::Stop))
        }
        Ok(None)
    }
}