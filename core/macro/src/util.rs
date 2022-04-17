pub(crate) fn prepend_crate(path: String) -> String {
    if !path.starts_with("crate::") {
        format!("crate::{}", path)
    } else {
        path
    }
}
