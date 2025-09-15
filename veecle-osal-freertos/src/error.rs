use veecle_freertos_integration::FreeRtosError;
use veecle_osal_api::Error;

pub fn into_veecle_os_error(error: FreeRtosError) -> Error {
    match error {
        FreeRtosError::OutOfMemory => Error::OutOfMemory,
        _ => Error::Unknown,
    }
}
