use std::fs::{File, OpenOptions};
use std::path::PathBuf;
use std::sync::OnceLock;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

pub static LOG_FILE: OnceLock<Arc<Mutex<LogFile>>> = OnceLock::new();

pub struct LogFile {
    path: PathBuf,
    size: u64,
    file: File,
}
#[allow(dead_code)]
impl LogFile {
    #[inline]
    pub fn path(&self) -> &PathBuf { &self.path }
    #[inline]
    pub fn path_mut(&mut self) -> &mut PathBuf { &mut self.path }
    #[inline]
    pub fn size(&self) -> u64 { self.size }
    #[inline]
    pub fn size_mut(&mut self) -> &mut u64 { &mut self.size }
    #[inline]
    pub fn file(&self) -> &File { &self.file }
    #[inline]
    pub fn file_mut(&mut self) -> &mut File { &mut self.file }
}

pub struct Log;
impl Log {
    pub fn init(path: &str, file_size: u64) {
        let log_path = PathBuf::from(&path);

        if let Some(dir) = log_path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }

        let file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&log_path);

        match file {
            Ok(file) => {
                let log_file = LogFile {
                    path: log_path,
                    size: file_size,
                    file,
                };
                LOG_FILE.get_or_init(|| Arc::new(Mutex::new(log_file)));
            },
            Err(e) => { eprintln!("{e}") }
        }
    }

    pub fn re_init(old_log_path: PathBuf) {
        let old_log_p =
            if let Some(p) = old_log_path.to_str() && !p.is_empty() { p }
            else { return };

        let p =
            if let Some(p) = Self::create_new_name(old_log_p) { p }
            else { return };
        let new_log_path = PathBuf::from(p);

        if let Some(dir) = PathBuf::from(&old_log_path).parent() {
            let _ = std::fs::create_dir_all(dir);
        }

        let new_file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&new_log_path);

        match new_file {
            Ok(new_file) =>
                if let Some(log_file) = LOG_FILE.get().cloned() &&
                   let Ok(mut old_log_file) = log_file.try_lock()
                {
                    let new_log_file = LogFile {
                        path: new_log_path,
                        size: old_log_file.size,
                        file: new_file,
                    };
                    *old_log_file = new_log_file;
                }
            Err(e) => { eprintln!("{e}"); }
        }
    }
    fn create_new_name(old_name: &str) -> Option<String> {
        let (base, ext) = old_name.rsplit_once(".")?;
        let base_wo_time = if let Some(b) = base.rsplit_once("-").map(|x| x.0) { b } else { base };

        let timestamp = {
            match SystemTime::now().duration_since(UNIX_EPOCH) {
                Ok(ts) => ts,
                Err(e) => {
                    eprintln!("{e}");
                    return None
                }
            }
                .as_nanos().to_string()
        };

        let new_name = format!("{base_wo_time}-{timestamp}.{ext}");
        Some(new_name)
    }

}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {{
        let mut meta_len = 0_u64;
        let mut log_file_size = 0_u64;

        if let Some(m) = $crate::logging::LOG_FILE.get() &&
		   let Ok(log_file) = m.try_lock() &&
           let Ok(meta) = log_file.file().metadata()
		{
            meta_len = meta.len();
            log_file_size = log_file.size();
        }

        if meta_len > log_file_size {
            let mut old_log_path = std::path::PathBuf::new();

            if let Some(m) = $crate::logging::LOG_FILE.get().cloned() &&
	           let Ok(log_file) = m.try_lock()
            {
                old_log_path = log_file.path().clone();
            }
            $crate::logging::Log::re_init(old_log_path);
        }

        if let Some(m) = $crate::logging::LOG_FILE.get().cloned() &&
		   let Ok(mut log_file) = m.try_lock()
		{
            let time = chrono::offset::Utc::now();
            let _ = log_file.file_mut().write_fmt(core::format_args!("[{}]: ", time));
            let _ = log_file.file_mut().write_fmt(core::format_args!($($arg)*));
            let _ = log_file.file_mut().write(b"\n");
        }
    }}
}
