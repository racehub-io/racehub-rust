/// Represents telemetry data for a single car
#[derive(Default, Clone)]
pub struct CarTelemetry {
    pub speed: Option<f32>,
    pub rpm: Option<f32>,
    pub throttle: Option<f32>,
    pub steer: Option<f32>,
    pub brake: Option<f32>,
    pub clutch: Option<f32>,
    pub gear: Option<i32>,
    pub drs: Option<u8>,
    pub brake_temperature_front_left: Option<u16>,
    pub brake_temperature_front_right: Option<u16>,
    pub brake_temperature_rear_left: Option<u16>,
    pub brake_temperature_rear_right: Option<u16>,
    pub tyre_surface_temperature_front_left: Option<u8>,
    pub tyre_surface_temperature_front_right: Option<u8>,
    pub tyre_surface_temperature_rear_left: Option<u8>,
    pub tyre_surface_temperature_rear_right: Option<u8>,
    pub tyre_carcass_temperature_front_left: Option<u8>,
    pub tyre_carcass_temperature_front_right: Option<u8>,
    pub tyre_carcass_temperature_rear_left: Option<u8>,
    pub tyre_carcass_temperature_rear_right: Option<u8>,
    pub tyre_pressure_front_left: Option<f32>,
    pub tyre_pressure_front_right: Option<f32>,
    pub tyre_pressure_rear_left: Option<f32>,
    pub tyre_pressure_rear_right: Option<f32>,
    pub engine_temperature: Option<u16>,
    pub world_position_x: Option<f32>,
    pub world_position_y: Option<f32>,
    pub world_position_z: Option<f32>,
    pub g_force_lateral: Option<f32>,
    pub g_force_longitudinal: Option<f32>,
    pub g_force_vertical: Option<f32>,
    pub yaw: Option<f32>,
    pub pitch: Option<f32>,
    pub roll: Option<f32>,
    pub lap_time: Option<f32>,
    pub lap_distance: Option<f32>,
}

/// Trait that each game platform must implement to convert their
/// specific packet types into our common telemetry format
pub trait TelemetryProvider {
    type Packet;
    type Error;

    /// Convert platform-specific packet into our common car telemetry format
    fn get_car_telemetry(&self, packet: &Self::Packet) -> Result<Vec<CarTelemetry>, Self::Error>;

    /// Return the number of cars in the session
    fn get_car_count(&self, packet: &Self::Packet) -> usize;
} 