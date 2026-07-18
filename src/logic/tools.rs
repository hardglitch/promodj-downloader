use crate::log;
use regex::Regex;
use std::io::Write;
use std::sync::LazyLock;
use std::time::{Duration, SystemTimeError, UNIX_EPOCH};

// Remove forbidden symbols
static RE1: LazyLock<Regex> = LazyLock::new(|| {
    match Regex::new(r#"[!$%*?\/\\:\"<>|]"#) {
        Ok(r) => r,
        Err(e) => {
            log!("RE1: {e}"); panic!("{e}");
        }
    }
});
// Remove double spaces
static RE2: LazyLock<Regex> = LazyLock::new(|| {
    match Regex::new(r" {2,}") {
        Ok(r) => r,
        Err(e) => {
            log!("RE2: {e}"); panic!("{e}");
        }
    }
});

pub fn clear_filename(filename: &str) -> String {
    let s = RE1.replace_all(filename, " ");
    let s = RE2.replace_all(&s, " ");
    s.trim().to_owned()
}
pub fn new_filename(old_filename: &str) -> Option<String> {
    let mut split = old_filename.rsplit('.');
    let name = split.next()?;
    let ext = split.next()?;
    let ts = std::time::SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();
    let new_filename = format!("{name}-{ts}.{ext}");
    Some(new_filename)
}
pub fn timestamp() -> u64 {
    match std::time::SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(ts) => ts.as_secs(),
        Err(e) => { log!("{e}"); 0 }
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clear_filename1_pos() {
        let f_name = " Some   && unstadrste  Name $$$ ?";
        let res = clear_filename(f_name);
        assert_eq!("Some && unstadrste Name", res);
    }
    #[test]
    fn test_clear_filename2_pos() {
        let f_name = "09 - 𝚣𝚘𝚗𝚎 ~𝒕𝒉𝒆 𝒊𝒏𝒕𝒆𝒓𝒔𝒕𝒊𝒄𝒆~";
        let res = clear_filename(f_name);
        assert_eq!("09 - 𝚣𝚘𝚗𝚎 ~𝒕𝒉𝒆 𝒊𝒏𝒕𝒆𝒓𝒔𝒕𝒊𝒄𝒆~", res);
    }
    #[test]
    fn test_clear_filename3_pos() {
        let f_name = "07 - 𝖂𝖎𝖓𝖙𝖊𝖗𝖋𝖆𝖚𝖓 & 𝑻𝒉𝒆 𝑴𝒂𝒄𝒉𝒊𝒏𝒆";
        let res = clear_filename(f_name);
        assert_eq!("07 - 𝖂𝖎𝖓𝖙𝖊𝖗𝖋𝖆𝖚𝖓 & 𝑻𝒉𝒆 𝑴𝒂𝒄𝒉𝒊𝒏𝒆", res);
    }
    #[test]
    fn test_new_filename1_pos() {
        let f_name = "07 - 𝖂𝖎𝖓𝖙𝖊𝖗𝖋𝖆𝖚𝖓 & 𝑻𝒉𝒆 𝑴𝒂𝒄𝒉𝒊𝒏𝒆";
        let res = new_filename(f_name);
        assert_ne!(Some("07 - 𝖂𝖎𝖓𝖙𝖊𝖗𝖋𝖆𝖚𝖓 & 𝑻𝒉𝒆 𝑴𝒂𝒄𝒉𝒊𝒏𝒆".to_owned()), res);
    }
    #[test]
    fn test_new_filename2_pos() {
        let f_name = "07 - 𝖂𝖎𝖓𝖙𝖊𝖗𝖋𝖆𝖚𝖓 & 𝑻𝒉𝒆 𝑴𝒂𝒄𝒉𝒊𝒏𝒆.flac";
        let res = new_filename(f_name);
        assert_ne!(Some("07 - 𝖂𝖎𝖓𝖙𝖊𝖗𝖋𝖆𝖚𝖓 & 𝑻𝒉𝒆 𝑴𝒂𝒄𝒉𝒊𝒏𝒆.flac".to_owned()), res);
    }
}