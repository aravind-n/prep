pub(crate) fn host_os() -> &'static str {
    if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(windows) {
        "windows"
    } else {
        "linux"
    }
}

pub(crate) fn expand_env(s: &str) -> String {
    shellexpand::env(s).unwrap_or_else(|_| s.into()).into()
}
