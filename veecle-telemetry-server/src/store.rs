use std::sync::Mutex;

use tokio::sync::Notify;

#[derive(Clone, Debug, Default)]
pub struct TracingLineData {
    pub lines: Vec<String>,
    pub done: bool,
}

#[derive(Debug, Default)]
pub struct TracingLineStore {
    data: Mutex<TracingLineData>,
    notify: Notify,
}

impl TracingLineStore {
    pub fn read<T>(&self, f: impl FnOnce(&TracingLineData) -> T) -> T {
        let data = self.data.lock().unwrap();

        f(&data)
    }

    fn write<T>(&self, f: impl FnOnce(&mut TracingLineData) -> T) -> T {
        let mut data = self.data.lock().unwrap();

        let result = f(&mut data);

        self.notify.notify_waiters();

        result
    }

    pub fn push_line(&self, line: String) -> usize {
        self.write(move |data| {
            data.lines.push(line);

            data.lines.len()
        })
    }

    pub fn set_done(&self) {
        self.write(move |data| {
            data.done = true;
        });
    }

    pub async fn wait_for_data(&self) {
        self.notify.notified().await;
    }
}
