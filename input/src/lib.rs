use nix::unistd::Uid;
use odilia_common::events::ScreenReaderEvent;
use std::{
    env,
    path::Path,
    process::{exit, id},
    time::{SystemTime, UNIX_EPOCH},
};
use sysinfo::{ProcessExt, System, SystemExt};
use tokio::{fs, io::AsyncReadExt, net::UnixListener, sync::broadcast, sync::mpsc::Sender};

fn get_log_file_name() -> String {
    let time = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_secs().to_string(),
        Err(_) => {
            tracing::error!("SystemTime before UnixEpoch!");
            exit(1);
        }
    };

    match env::var("XDG_DATA_HOME") {
        Ok(val) => {
            tracing::info!(
                "XDG_DATA_HOME Variable is present, using it's value for default file path."
            );
            format!("{val}/sohks/sohks-{time}.log")
        }
        Err(e) => {
            tracing::trace!(
                "XDG_DATA_HOME Variable is not set, falling back on hardcoded path.\nError: {:#?}",
                e
            );

            format!("~/.local/share/sohks/sohks-{time}.log")
        }
    }
}

pub async fn sr_event_receiver(
    event_sender: Sender<ScreenReaderEvent>,
    shutdown_rx: &mut broadcast::Receiver<i32>,
) -> eyre::Result<()> {
    //tracing::trace!("Setting process umask.");
    //umask(Mode::S_IWGRP | Mode::S_IWOTH);

    let (pid_file_path, sock_file_path) = get_file_paths();
    let log_file_name = get_log_file_name();

    let log_path = Path::new(&log_file_name);
    tracing::debug!("Socket file located at: {:?}", sock_file_path);
    if let Some(p) = log_path.parent() {
        if !p.exists() {
            if let Err(e) = fs::create_dir_all(p).await {
                tracing::error!("Failed to create log dir: {}", e);
            }
        }
    }

    if Path::new(&pid_file_path).exists() {
        tracing::trace!("Reading {} file and checking for running instances.", pid_file_path);
        let sohkds_pid = match fs::read_to_string(&pid_file_path).await {
            Ok(sohkds_pid) => sohkds_pid,
            Err(e) => {
                tracing::error!("Unable to read {} to check all running instances", e);
                exit(1);
            }
        };
        tracing::debug!("Previous PID: {}", sohkds_pid);

        let mut sys = System::new_all();
        sys.refresh_all();
        for (pid, process) in sys.processes() {
            if pid.to_string() == sohkds_pid && process.exe() == env::current_exe().unwrap() {
                tracing::error!("Server is already running!");
                exit(1);
            }
        }
    }

    if Path::new(&sock_file_path).exists() {
        tracing::trace!("Sockfile exists, attempting to remove it.");
        match fs::remove_file(&sock_file_path).await {
            Ok(_) => {
                tracing::debug!("Removed old socket file");
            }
            Err(e) => {
                tracing::error!("Error removing the socket file!: {}", e);
                tracing::error!("You can manually remove the socket file: {}", sock_file_path);
                exit(1);
            }
        };
    }

    match fs::write(&pid_file_path, id().to_string()).await {
        Ok(_) => {}
        Err(e) => {
            tracing::error!("Unable to write to {}: {}", pid_file_path, e);
            exit(1);
        }
    }

    let listener = UnixListener::bind(sock_file_path).expect("Could not open socket");
    tracing::debug!("Listener activated!");
    loop {
        tokio::select! {
            msg = listener.accept() => {
                match msg {
                    Ok((mut socket, address)) => {
                        tracing::debug!("Ok from socket");
                        let mut response = String::new();
                        match socket.read_to_string(&mut response).await {
                          Ok(_) => {},
                          Err(e) => {
                            tracing::error!("Error reading from socket {:#?}", e);
                          }
                        }
                        // if valid screen reader event
                        match serde_json::from_str::<ScreenReaderEvent>(&response) {
                          Ok(sre) => {
                            match  event_sender.send(sre).await {
                              Err(err) =>  tracing::error!("Error sending ScreenReaderEvent over socket: {}", err),
                              Ok(_) => tracing::debug!("Sent SR event"),
                            }
                          },
                          Err(e) => tracing::debug!("Invalid odilia event. {:#?}", e)
                        }
                        tracing::debug!("Socket: {:?} Address: {:?} Response: {}", socket, address, response);
                    },
                    Err(e) => tracing::error!("accept function failed: {:?}", e),
                }
                continue;
            }
            _ = shutdown_rx.recv() => {
                tracing::debug!("Shutting down input socker.");
                break;
            }
        }
    }
    Ok(())
}

fn get_file_paths() -> (String, String) {
    match env::var("XDG_RUNTIME_DIR") {
        Ok(val) => {
            tracing::info!(
                "XDG_RUNTIME_DIR Variable is present, using it's value as default file path."
            );

            let pid_file_path = format!("{val}/sohkds.pid");
            let sock_file_path = format!("{val}/sohkd.sock");

            (pid_file_path, sock_file_path)
        }
        Err(e) => {
            tracing::trace!("XDG_RUNTIME_DIR Variable is not set, falling back on hardcoded path.\nError: {:#?}", e);

            let pid_file_path = format!("/run/user/{}/sohkds.pid", Uid::current());
            let sock_file_path = format!("/run/user/{}/sohkd.sock", Uid::current());

            (pid_file_path, sock_file_path)
        }
    }
}
