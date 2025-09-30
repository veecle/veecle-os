/// Test that the macro works when veecle-telemetry is renamed.

// Synchronous functions:
#[my_veecle_telemetry::instrument]
pub fn sync_basic() {
    unimplemented!("testing compilation")
}

#[my_veecle_telemetry::instrument(short_name = true)]
pub fn sync_short_name() {
    unimplemented!("testing compilation")
}

#[my_veecle_telemetry::instrument(name = "custom_name")]
pub fn sync_custom_name() {
    unimplemented!("testing compilation")
}

#[my_veecle_telemetry::instrument(properties = { "key": "value", "number": 42 })]
pub fn sync_properties() {
    unimplemented!("testing compilation")
}

#[my_veecle_telemetry::instrument(short_name = true, properties = { "param": "value" })]
pub fn sync_short_name_and_properties() {
    unimplemented!("testing compilation")
}

// Asynchronous functions:
#[my_veecle_telemetry::instrument]
pub async fn async_basic() {
    unimplemented!("testing compilation")
}

#[my_veecle_telemetry::instrument(short_name = true)]
pub async fn async_short_name() {
    unimplemented!("testing compilation")
}

#[my_veecle_telemetry::instrument(name = "custom_async")]
pub async fn async_custom_name() {
    unimplemented!("testing compilation")
}

#[my_veecle_telemetry::instrument(properties = { "key": "value", "count": 10 })]
pub async fn async_properties() {
    unimplemented!("testing compilation")
}

#[my_veecle_telemetry::instrument(short_name = true, properties = { "id": 123 })]
pub async fn async_short_name_and_properties() {
    unimplemented!("testing compilation")
}
