use std::ptr::NonNull;
use std::{ffi::CStr, path::Path};

use tsk_sys::TSK_FS_TYPE_ENUM_TSK_FS_TYPE_DETECT;

pub fn get_tsk_version() -> String {
    let cstr = unsafe { CStr::from_ptr(tsk_sys::tsk_version_get_str()) };
    String::from_utf8_lossy(cstr.to_bytes()).to_string()
}

struct TskPath {
    path: Vec<u8>,
}

#[derive(Debug)]
pub struct TskError {
    message: String,
}

impl std::fmt::Display for TskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for TskError {}

pub type TskResult<T> = Result<T, TskError>;

impl TskPath {
    #[cfg(target_family = "unix")]
    fn from_path(path: &Path) -> Self {
        use std::os::unix::ffi::OsStrExt as _;
        Self {
            path: path.as_os_str().as_bytes().into(),
        }
    }

    #[cfg(target_os = "windows")]
    fn from_path(path: &Path) -> Self {
        let path_wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
        let path = unsafe {
            std::slice::from_raw_parts(path_wide.as_ptr() as *const u8, v.len() * 2).to_vec()
        };
        Self { path }
    }

    fn as_ptr(&self) -> *const i8 {
        self.path.as_ptr() as *const i8
    }
}

fn try_get_tsk_error() -> TskError {
    let message_ptr = unsafe { tsk_sys::tsk_error_get() };
    if message_ptr.is_null() {
        return TskError {
            message: String::from("unknown"),
        };
    }
    let message = unsafe { CStr::from_ptr(message_ptr) }
        .to_string_lossy()
        .to_string();
    TskError { message }
}

fn get_tsk_result<T>(result: *mut T) -> Result<NonNull<T>, TskError> {
    NonNull::new(result).ok_or_else(try_get_tsk_error)
}

pub struct TskImage {
    pub(crate) inner: NonNull<tsk_sys::TSK_IMG_INFO>,
}

impl TskImage {
    pub fn open(path: &Path) -> TskResult<Self> {
        let tsk_path = TskPath::from_path(path);
        let tsk_img_result = unsafe {
            tsk_sys::tsk_img_open_sing(
                tsk_path.as_ptr(),
                tsk_sys::TSK_IMG_TYPE_ENUM_TSK_IMG_TYPE_RAW_SING,
                0,
            )
        };
        get_tsk_result(tsk_img_result).map(|inner| Self { inner })
    }

    pub fn open_fs(&mut self) -> TskResult<TskFs> {
        let tsk_fs_result = unsafe {
            tsk_sys::tsk_fs_open_img(self.inner.as_mut(), 0, TSK_FS_TYPE_ENUM_TSK_FS_TYPE_DETECT)
        };
        get_tsk_result(tsk_fs_result).map(|inner| TskFs { inner })
    }
}

pub struct TskFs {
    pub(crate) inner: NonNull<tsk_sys::TSK_FS_INFO>,
}

impl TskFs {
    pub fn get_fs_type(&self) -> TskResult<String> {
        let ty = unsafe { (*self.inner.as_ptr()).ftype };
        let name_ptr = unsafe { tsk_sys::tsk_fs_type_toname(ty) };
        get_tsk_result(name_ptr as _)
            .map(|non_null| unsafe { CStr::from_ptr(non_null.as_ptr()).to_bytes() })
            .map(|bytes| String::from_utf8_lossy(bytes).to_string())
    }
}

#[cfg(test)]
mod test {
    use std::fs::File;
    use std::io::{Read as _, Write as _};

    use tempfile::NamedTempFile;

    use super::*;
    #[test]
    fn test_get_tsk_version() {
        assert_eq!(get_tsk_version(), "4.12.1");
    }

    #[test]
    fn test_ntfs() {
        let source = "test_data/smol.ntfs.gz";
        let file = File::open(source).expect("Failed to load test data");
        let mut gz = flate2::read::GzDecoder::new(file);
        let mut ntfs_raw = Vec::new();
        gz.read_to_end(&mut ntfs_raw)
            .expect("Failed to read test data");
        let mut tempfile = NamedTempFile::new().expect("Failed to open tempfile");
        tempfile.write(&ntfs_raw).expect("Failed to write tempfile");
        let mut image = TskImage::open(tempfile.path()).expect("Failed to open ntfs image");
        let fs = image.open_fs().expect("Failed to open NTFS FS");
        eprintln!("{}", fs.get_fs_type().expect("Failed to get NTFS FS type"));
    }
}
