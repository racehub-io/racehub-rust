mod f12024;
mod db;
mod telemetry;
mod recorder;
pub mod native;
pub mod protocols;

use std::ffi::c_void;
use std::panic::catch_unwind;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;
use crate::db::DB;
use crate::recorder::RECORDER;

// Re-export the main types for easier access
pub use f12024::{TelemetryCapture, TelemetryError, TelemetryPacket};
pub use f12024::telemetry::{F12024Provider, F12024Metadata};
pub use common::f12024::F1TelemetryPacket;
pub use telemetry::{CarTelemetry, TelemetryProvider};
pub use recorder::{TelemetryReader, PacketInfo};

// Opaque type to represent the TelemetryCapture instance
#[repr(C)]
pub struct TelemetryCaptureHandle(*mut c_void);

// Safe wrapper around the callback and user data
struct CallbackWrapper {
    callback: PacketCallback,
    user_data: *mut c_void,
}

// Mark the wrapper as Send + Sync since we ensure thread-safe access
unsafe impl Send for CallbackWrapper {}
unsafe impl Sync for CallbackWrapper {}

type PacketCallback = extern "C" fn(*mut c_void, *const u8, usize);

#[unsafe(no_mangle)]
pub extern "C" fn create_telemetry_capture(port: u16, callback: PacketCallback, user_data: *mut c_void) -> *mut TelemetryCaptureHandle {
    let result = catch_unwind(|| {
        // Create the runtime
        let rt = Runtime::new().expect("Failed to create Tokio runtime");
        
        // Wrap the callback and user data in a thread-safe container
        let callback_wrapper = Arc::new(Mutex::new(CallbackWrapper {
            callback,
            user_data,
        }));
        
        // Create the capture instance
        let capture = TelemetryCapture::new(port, move |packet| {
            // Convert the packet to bytes for FFI
            let bytes = bincode::serialize(&packet).expect("Failed to serialize packet");
            
            // Call the callback in a thread-safe way
            if let Ok(wrapper) = callback_wrapper.lock() {
                (wrapper.callback)(wrapper.user_data, bytes.as_ptr(), bytes.len());
            }
        });

        // Box both the runtime and capture together
        let handle = Box::new((rt, capture));
        Box::into_raw(handle) as *mut TelemetryCaptureHandle
    });

    match result {
        Ok(handle) => handle,
        Err(_) => std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn start_telemetry_capture(handle: *mut TelemetryCaptureHandle) -> bool {
    if handle.is_null() {
        return false;
    }

    let result = catch_unwind(|| {
        let handle_ptr = handle as *mut (Runtime, TelemetryCapture);
        let (rt, capture) = unsafe { &mut *handle_ptr };

        // Run the capture in the runtime
        rt.block_on(async {
            if let Err(e) = capture.run().await {
                eprintln!("Telemetry capture error: {:?}", e);
                return false;
            }
            true
        })
    });

    result.unwrap_or(false)
}

#[unsafe(no_mangle)]
pub extern "C" fn destroy_telemetry_capture(handle: *mut TelemetryCaptureHandle) {
    if !handle.is_null() {
        let result = catch_unwind(|| {
            let handle_ptr = handle as *mut (Runtime, TelemetryCapture);
            unsafe {
                drop(Box::from_raw(handle_ptr));
            }
        });

        if let Err(e) = result {
            eprintln!("Error destroying telemetry capture: {:?}", e);
        }
        
        // Finalize telemetry recordings
        if let Ok(mut recorder) = RECORDER.lock() {
            if let Err(e) = recorder.finalize() {
                eprintln!("Error finalizing telemetry recording: {:?}", e);
            }
        }
        
        // Always flush and close the DB when destroying a telemetry capture
        flush_and_close_db();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn flush_and_close_db() {
    if let Ok(mut db) = DB.lock() {
        if let Err(e) = db.flush_and_close() {
            eprintln!("Error flushing and closing database: {}", e);
        }
    }
}
