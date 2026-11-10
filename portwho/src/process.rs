use crate::utils::{Pid, Process, Processes};

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub fn parse_socket_inode(target: &Path) -> Option<u64> {
    target
        .to_str()?
        .strip_prefix("socket:[")?
        .strip_suffix(']')?
        .parse::<u64>()
        .ok()
}

fn read_cmdline(path: &Path) -> String {
    let data = match fs::read(path) {
        Ok(data) => data,
        Err(_) => return String::new(),
    };

    data.split(|&b| b == 0)
        .filter(|part| !part.is_empty())
        .map(|part| String::from_utf8_lossy(part).into_owned())
        .collect::<Vec<_>>()
        .join(" ")
}

fn read_process_from_path(pid: Pid, base: &Path) -> io::Result<Process> {
    let fds = fs::read_dir(base.join("fd"))?;

    let bin = fs::read_link(base.join("exe"))
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_default();

    let cmdline = read_cmdline(&base.join("cmdline"));

    let mut socket_inodes = Vec::new();

    for fd in fds.flatten() {
        if let Ok(target) = fs::read_link(fd.path()) {
            if let Some(inode) = parse_socket_inode(&target) {
                socket_inodes.push(inode);
            }
        }
    }

    socket_inodes.sort_unstable();
    socket_inodes.dedup();

    Ok(Process {
        pid,
        bin,
        cmdline,
        socket_inodes,
    })
}

pub fn get_process(pid: Pid) -> Result<Process, String> {
    let path = PathBuf::from(format!("/proc/{pid}"));

    if !path.exists() {
        return Err(format!("process {pid} does not exist"));
    }

    read_process_from_path(pid, &path)
        .map_err(|e| format!("cannot inspect process {pid} (permissions or process exited): {e}"))
}

pub fn get_processes() -> Result<Processes, String> {
    let entries = fs::read_dir("/proc").map_err(|e| e.to_string())?;

    let mut processes = Processes::default();

    for entry in entries.flatten() {
        let pid = match entry.file_name().to_string_lossy().parse::<Pid>() {
            Ok(pid) => pid,
            Err(_) => continue,
        };

        if let Ok(process) = read_process_from_path(pid, &entry.path()) {
            if !process.socket_inodes.is_empty() {
                processes.insert(pid, process);
            }
        }
    }

    Ok(processes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_fd_symlink() {
        assert_eq!(parse_socket_inode(Path::new("socket:[90210]")), Some(90210));

        assert_eq!(parse_socket_inode(Path::new("pipe:[90210]")), None);

        assert_eq!(parse_socket_inode(Path::new("socket:[bad]")), None);
    }
}
