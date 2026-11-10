use crate::utils::{NetSockets, Processes};

pub fn match_sockets_processes(sockets: &mut NetSockets, processes: &Processes) {
    for process in processes.values() {
        for inode in &process.socket_inodes {
            if let Some(socket) = sockets.get_mut(inode) {
                socket.process_pids.push(process.pid);
            }
        }
    }
}
