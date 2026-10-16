use std::ffi::c_void;
use libloading::Library;

extern "C" fn packet_callback(_user_data: *mut c_void, _packet_data: *const u8, _packet_size: usize) {
    // unsafe {
    //     // Convert the raw packet data back into our packet type
    //     let packet_slice = std::slice::from_raw_parts(packet_data, packet_size);
        
    //     if let Ok(packet) = bincode::deserialize::<F1TelemetryPacket>(packet_slice) {
    //         match packet {
    //             F1TelemetryPacket::Event(event_packet) => {
    //                 println!("Received Event Packet: {:?}", event_packet);
    //             }
    //             F1TelemetryPacket::CarTelemetry(car_telemetry_packet) => {
    //                 println!("Received Car Telemetry Packet: {:?}", car_telemetry_packet);
    //             }
    //             F1TelemetryPacket::Unknown(header) => {
    //                 println!("Received Unknown Packet with ID {}: {:?}", header.m_packetId, header);
    //             }
    //         }
            
    //         // You can also access the user data if needed
    //         let _user_context = user_data as *mut u32;
    //     } else {
    //         eprintln!("Failed to deserialize packet");
    //     }
    // }
}

#[tokio::main]
async fn main() {
    // Initialize the library
    let lib = unsafe {
        Library::new("D:/Github_Workspaces/racehub-rust/target/debug/capture.dll")
            .expect("Could not load library")
    };

    // Load the functions from the library (note the added *)
    let create_telemetry_capture: unsafe extern "C" fn(u16, extern "C" fn(*mut c_void, *const u8, usize), *mut c_void) -> *mut c_void = unsafe {
        *lib.get(b"create_telemetry_capture").expect("Could not load create_telemetry_capture")
    };
    
    let start_telemetry_capture: unsafe extern "C" fn(*mut c_void) -> bool = unsafe {
        *lib.get(b"start_telemetry_capture").expect("Could not load start_telemetry_capture")
    };
    
    let destroy_telemetry_capture: unsafe extern "C" fn(*mut c_void) = unsafe {
        *lib.get(b"destroy_telemetry_capture").expect("Could not load destroy_telemetry_capture")
    };

    let mut user_context: u32 = 42;

    println!("Creating telemetry capture...");
    
    // Create the capture instance
    let handle = unsafe {
        create_telemetry_capture(
            30500,
            packet_callback,
            &mut user_context as *mut u32 as *mut c_void,
        )
    };

    if handle.is_null() {
        eprintln!("Failed to create telemetry capture!");
        return;
    }

    println!("Starting capture on port 30500...");
    
    // Start capturing
    let success = unsafe { start_telemetry_capture(handle) };
    
    if !success {
        eprintln!("Failed to start telemetry capture!");
        unsafe { destroy_telemetry_capture(handle) };
        return;
    }

    // The capture runs until the process is terminated
    println!("Capture running. Press Ctrl+C to stop.");
    
    // Wait for Ctrl+C
    tokio::signal::ctrl_c().await.unwrap();
    
    println!("\nShutting down...");

    // Clean up - this will also flush and close the DB
    unsafe {
        destroy_telemetry_capture(handle);
    }
} 