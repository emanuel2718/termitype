use crate::{
    constants::db_file,
    db::{Db, LeaderboardResult},
    log_error,
};
use std::{
    sync::mpsc::{self, RecvTimeoutError, SyncSender, TrySendError},
    thread::JoinHandle,
    time::Duration,
};

const WRITER_CHANNEL_CAPACITY: usize = 256;

enum WriteMessage {
    Save(LeaderboardResult),
    Shutdown,
}

pub enum EnqueueError {
    Full(LeaderboardResult),
    Disconnected(LeaderboardResult),
}

pub struct DbWriter {
    sender: SyncSender<WriteMessage>,
    join_handle: Option<JoinHandle<()>>,
}

impl Default for DbWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl DbWriter {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::sync_channel::<WriteMessage>(WRITER_CHANNEL_CAPACITY);

        let join_handle = std::thread::spawn(move || {
            let mut db = match Db::new(db_file()) {
                Ok(db) => db,
                Err(err) => {
                    log_error!("DB writer: failed to initialize background db: {err}");
                    return;
                }
            };

            loop {
                match receiver.recv_timeout(Duration::from_millis(200)) {
                    Ok(WriteMessage::Save(result)) => {
                        if let Err(err) = db.write_result(result) {
                            log_error!("DB writer: failed writing result: {err}");
                        }
                    }
                    Ok(WriteMessage::Shutdown) => break,
                    Err(RecvTimeoutError::Timeout) => {}
                    Err(RecvTimeoutError::Disconnected) => break,
                }
            }
        });

        Self {
            sender,
            join_handle: Some(join_handle),
        }
    }

    pub fn enqueue(&self, result: LeaderboardResult) -> Result<(), EnqueueError> {
        match self.sender.try_send(WriteMessage::Save(result)) {
            Ok(()) => Ok(()),
            Err(TrySendError::Full(WriteMessage::Save(result))) => Err(EnqueueError::Full(result)),
            Err(TrySendError::Disconnected(WriteMessage::Save(result))) => {
                Err(EnqueueError::Disconnected(result))
            }
            Err(TrySendError::Full(WriteMessage::Shutdown))
            | Err(TrySendError::Disconnected(WriteMessage::Shutdown)) => {
                unreachable!("shutdown message should not be produced by enqueue")
            }
        }
    }

    pub fn shutdown(&mut self) {
        let _ = self.sender.send(WriteMessage::Shutdown);
        if let Some(join_handle) = self.join_handle.take() {
            let _ = join_handle.join();
        }
    }
}

impl Drop for DbWriter {
    fn drop(&mut self) {
        self.shutdown();
    }
}
