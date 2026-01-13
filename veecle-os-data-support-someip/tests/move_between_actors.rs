//! Tests whether a deserialized payload can be moved between actors without copy.

use std::convert::Infallible;
use veecle_os_data_support_someip::header::*;
use veecle_os_data_support_someip::parse::ParseExt;
use veecle_os_data_support_someip::service_discovery;
use veecle_os_data_support_someip::service_discovery::{Entry, ServiceEntry};
use veecle_os_runtime::actor;
use veecle_os_runtime::memory_pool::{Chunk, MemoryPool};
use veecle_os_runtime::{ExclusiveReader, Storable, Writer};
use yoke::{Yoke, Yokeable};

#[test]
fn yoke() {
    // Raw SOME/IP service discovery message.
    // First 16 bytes is a header, rest is a service discovery header.
    const BYTES: &[u8; 84] = &[
        255, 128, 129, 0, 0, 0, 0, 76, 0, 0, 20, 147, 1, 1, 2, 0, 64, 0, 0, 0, 0, 0, 0, 32, 1, 0,
        0, 16, 3, 232, 0, 10, 1, 0, 0, 128, 0, 0, 0, 0, 1, 1, 0, 16, 3, 235, 0, 10, 1, 0, 0, 128,
        0, 0, 0, 0, 0, 0, 0, 24, 0, 9, 4, 0, 192, 0, 2, 0, 0, 17, 0, 24, 0, 9, 4, 0, 192, 0, 2, 0,
        0, 17, 0, 26,
    ];

    static POOL: MemoryPool<[u8; 84], 5> = MemoryPool::new();

    #[derive(Debug)]
    pub struct Input;

    impl Storable for Input {
        type DataType = Chunk<'static, [u8; 84]>;
    }

    #[derive(Debug)]
    pub struct Output;

    impl Storable for Output {
        type DataType = Yoke<YokeWrapper<'static>, Chunk<'static, [u8; 84]>>;
    }

    #[derive(Debug, Yokeable)]
    pub struct YokeWrapper<'a>(service_discovery::Header<'a>);

    #[actor]
    async fn deserializer(
        mut input: ExclusiveReader<'_, Input>,
        mut writer: Writer<'_, Output>,
    ) -> Infallible {
        loop {
            input.wait_for_update().await;
            let Some(bytes) = input.take() else { continue };
            let yoked: Yoke<YokeWrapper, Chunk<'static, [u8; 84]>> =
                Yoke::attach_to_cart(bytes, |bytes| {
                    let (_header, payload) = Header::parse_with_payload(bytes.as_ref())
                        .expect("failed to parse SOME/IP message");

                    YokeWrapper(
                        service_discovery::Header::parse(payload.into_inner())
                            .expect("failed to deserialize SOME/IP service discovery header"),
                    )
                });

            writer.write(yoked).await;
        }
    }

    veecle_os_test::block_on_future(veecle_os_test::execute! {
        store: [Input, Output],
        actors: [
            Deserializer,
        ],
        validation: async |mut writer: Writer<'a, Input>, mut reader: ExclusiveReader<'a, Output>| {
            async {
                let chunk = POOL.chunk(*BYTES).unwrap();
                writer.write(chunk).await;
                let deserialized = reader.wait_for_update().await.take().unwrap();
                let expected_entry = Entry::OfferService(
                    ServiceEntry {
                        first_option: 0x00,
                        second_option: 0x00,
                        option_counts: 16,
                        service_id: 0x03E8,
                        instance_id: 0x000A,
                        major_version_ttl: 0x1000080,
                        minor_version: 0,
                    }
                );
                assert_eq!(
                    deserialized.get().0.entries.iter().next().unwrap(),
                    expected_entry
                );
            }.await;
        }
    });
}
