use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc;
use std::time::Duration;

use chordkit_file::{ByteLimit, ReadError, SETTINGS_FILE_LIMIT, read_bytes, read_text};

use crate::scratch::Scratch;

const SMALL: ByteLimit = ByteLimit::new(16);
const HANG_AFTER: Duration = Duration::from_secs(10);

#[test]
fn a_normal_file_reads_as_before() {
    let dir = Scratch::new("normal");
    let path = dir.file("kdeglobals", "[Shortcuts]\nCopy=Ctrl+C\n".as_bytes());
    let text = read_text(&path, SETTINGS_FILE_LIMIT).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(text, "[Shortcuts]\nCopy=Ctrl+C\n");
}

#[test]
fn a_file_of_exactly_the_limit_is_read_and_one_more_byte_is_not() {
    let dir = Scratch::new("edge");
    let exact = dir.file("exact", &[b'a'; 16]);
    let over = dir.file("over", &[b'a'; 17]);
    assert_eq!(read_bytes(&exact, SMALL).map(|b| b.len()).ok(), Some(16));
    assert!(matches!(
        read_bytes(&over, SMALL),
        Err(ReadError::TooLarge(limit)) if limit == SMALL
    ));
}

#[test]
fn an_oversize_file_is_refused() {
    let dir = Scratch::new("oversize");
    let path = dir.file("big", &[b'#'; 4096]);
    assert!(matches!(
        read_text(&path, SMALL),
        Err(ReadError::TooLarge(_))
    ));
}

#[test]
fn a_file_that_is_not_utf8_is_refused() {
    let dir = Scratch::new("binary");
    let path = dir.file("binary", &[0xff, 0xfe, 0x00, 0x80]);
    assert!(matches!(
        read_text(&path, SETTINGS_FILE_LIMIT),
        Err(ReadError::NotText)
    ));
    assert!(read_bytes(&path, SETTINGS_FILE_LIMIT).is_ok());
}

#[test]
fn a_missing_file_is_missing() {
    let dir = Scratch::new("missing");
    let error = read_text(&dir.path().join("nothing"), SETTINGS_FILE_LIMIT).err();
    assert!(error.is_some_and(|e| e.is_missing()));
}

#[test]
fn a_folder_is_refused() {
    let dir = Scratch::new("folder");
    assert!(matches!(
        read_text(dir.path(), SETTINGS_FILE_LIMIT),
        Err(ReadError::NotRegular)
    ));
}

#[test]
fn a_device_is_refused_without_reading_it() {
    assert!(matches!(
        read_bytes(Path::new("/dev/zero"), SETTINGS_FILE_LIMIT),
        Err(ReadError::NotRegular)
    ));
}

#[cfg(unix)]
#[test]
fn a_link_is_followed_and_its_target_checked() {
    let dir = Scratch::new("link");
    let target = dir.file("real", b"ok");
    let to_file = dir.path().join("to-file");
    let to_folder = dir.path().join("to-folder");
    let dangling = dir.path().join("dangling");
    for (link, to) in [
        (&to_file, target.as_path()),
        (&to_folder, dir.path()),
        (&dangling, Path::new("/nonexistent/chordkit-file")),
    ] {
        std::os::unix::fs::symlink(to, link).unwrap_or_else(|e| panic!("{}: {e}", link.display()));
    }
    assert_eq!(
        read_text(&to_file, SETTINGS_FILE_LIMIT).ok().as_deref(),
        Some("ok")
    );
    assert!(matches!(
        read_text(&to_folder, SETTINGS_FILE_LIMIT),
        Err(ReadError::NotRegular)
    ));
    assert!(matches!(
        read_text(&dangling, SETTINGS_FILE_LIMIT),
        Err(ReadError::Missing)
    ));
}

/// A pipe with no writer would stop a plain `open` for good. The read runs on its own thread so
/// that a regression fails this test after a timeout rather than hanging the run.
#[cfg(unix)]
#[test]
fn a_pipe_is_refused_without_blocking() {
    let dir = Scratch::new("pipe");
    let pipe = dir.path().join("pipe");
    let made = Command::new("mkfifo").arg(&pipe).status();
    assert!(
        made.is_ok_and(|status| status.success()),
        "mkfifo is needed for this test"
    );
    assert!(matches!(
        read_without_hanging(pipe.clone()),
        Some(Err(ReadError::NotRegular))
    ));
    let link = dir.path().join("link-to-pipe");
    std::os::unix::fs::symlink(&pipe, &link).unwrap_or_else(|e| panic!("{e}"));
    assert!(matches!(
        read_without_hanging(link),
        Some(Err(ReadError::NotRegular))
    ));
}

fn read_without_hanging(path: PathBuf) -> Option<Result<String, ReadError>> {
    let (done, result) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = done.send(read_text(&path, SETTINGS_FILE_LIMIT));
    });
    result.recv_timeout(HANG_AFTER).ok()
}
