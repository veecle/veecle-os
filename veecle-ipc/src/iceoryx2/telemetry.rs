use std::sync::mpsc;
use std::thread;

use iceoryx2::node::Node;
use iceoryx2::port::unable_to_deliver_strategy::UnableToDeliverStrategy;
use iceoryx2::service::ipc;
use iceoryx2::service::service_name::ServiceName;
use veecle_ipc_protocol::Uuid;
use veecle_telemetry::collector::Export;
use veecle_telemetry::protocol::InstanceMessage;

/// An [`Export`] implementer that forwards telemetry messages via [`iceoryx2`].
#[derive(Debug)]
pub struct Exporter {
    sender: mpsc::Sender<Vec<u8>>,
}

impl Exporter {
    /// Creates a new [`iceoryx2`] telemetry exporter.
    ///
    /// Spawns a background thread that handles publishing the messages.
    pub(crate) fn new(node: &Node<ipc::Service>, runtime_id: Uuid) -> Self {
        let service_name =
            ServiceName::new(&format!("veecle/runtime/{runtime_id}/telemetry")).unwrap();
        let service = node
            .service_builder(&service_name)
            .publish_subscribe::<[u8]>()
            .open_or_create()
            .unwrap();

        let (sender, receiver) = mpsc::channel::<Vec<u8>>();

        thread::spawn(move || {
            let publisher = service
                .publisher_builder()
                .max_loaned_samples(4096)
                .initial_max_slice_len(4096)
                .unable_to_deliver_strategy(UnableToDeliverStrategy::Block)
                .create()
                .unwrap();

            // No idea what it's doing, but it seems it needs some time before sending the first
            // message or it will be missed by the subscriber.
            std::thread::sleep(std::time::Duration::from_millis(100));

            while let Ok(json) = receiver.recv() {
                match publisher.loan_slice_uninit(json.len()) {
                    Ok(sample) => {
                        let sample = sample.write_from_slice(&json);
                        let _ = sample.send();
                    }
                    Err(err) => {
                        eprintln!("failed to acquire slice: {err}");
                    }
                }

                // `Block` doesn't seem to block, without a delay here messages get dropped.
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        });

        Self { sender }
    }
}

impl Export for Exporter {
    /// Exports a telemetry message by forwarding it via [`iceoryx2`].
    fn export(&self, message: InstanceMessage<'_>) {
        let Ok(json) = serde_json::to_vec(&message) else {
            return;
        };
        let _ = self.sender.send(json);
    }
}
