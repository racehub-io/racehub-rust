use rusqlite::{Connection, Result as SqlResult};
use std::sync::Mutex;
use lazy_static::lazy_static;
use std::collections::HashMap;
use crate::telemetry::CarTelemetry;

pub struct DbHandler {
    conn: Option<Connection>,
    current_frame: Option<u32>,
    frame_cache: HashMap<u32, Vec<CarTelemetry>>,
    session_id: Option<u64>,
    session_time: Option<f32>,
}

// Static instance
lazy_static! {
    pub static ref DB: Mutex<DbHandler> = Mutex::new(DbHandler::new().expect("Failed to create database"));
}

impl DbHandler {
    fn new() -> SqlResult<Self> {
        Ok(DbHandler {
            conn: None,
            current_frame: None,
            frame_cache: HashMap::new(),
            session_id: None,
            session_time: None,
        })
    }
    
    // Helper method to check if connection exists
    fn has_connection(&self) -> bool {
        self.conn.is_some()
    }
    
    // Initialize database with a specific filename
    pub fn initialize_db(&mut self, db_path: &str) -> SqlResult<()> {
        // Close any existing connection
        self.conn = None;
        
        // Open new database connection
        let conn = Connection::open(db_path)?;
        
        // Set performance-oriented PRAGMAs
        conn.pragma_update(None, "journal_mode", &"WAL")?;
        conn.pragma_update(None, "temp_store", &"MEMORY")?;
        conn.pragma_update(None, "cache_size", &"10000")?;
        conn.pragma_update(None, "synchronous", &"NORMAL")?;
        
        // Create the telemetry table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS telemetry (
                session_id TEXT NOT NULL,
                frame_identifier INTEGER NOT NULL,
                car_index INTEGER NOT NULL,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                session_time REAL,
                speed REAL,
                rpm REAL,
                throttle REAL,
                steer REAL,
                brake REAL,
                clutch REAL,
                gear INTEGER,
                drs INTEGER,
                brake_temperature_front_left INTEGER,
                brake_temperature_front_right INTEGER,
                brake_temperature_rear_left INTEGER,
                brake_temperature_rear_right INTEGER,
                tyre_surface_temperature_front_left INTEGER,
                tyre_surface_temperature_front_right INTEGER,
                tyre_surface_temperature_rear_left INTEGER,
                tyre_surface_temperature_rear_right INTEGER,
                tyre_carcass_temperature_front_left INTEGER,
                tyre_carcass_temperature_front_right INTEGER,
                tyre_carcass_temperature_rear_left INTEGER,
                tyre_carcass_temperature_rear_right INTEGER,
                tyre_pressure_front_left REAL,
                tyre_pressure_front_right REAL,
                tyre_pressure_rear_left REAL,
                tyre_pressure_rear_right REAL,
                engine_temperature INTEGER,
                world_position_x REAL,
                world_position_y REAL,
                world_position_z REAL,
                g_force_lateral REAL,
                g_force_longitudinal REAL,
                g_force_vertical REAL,
                yaw REAL,
                pitch REAL,
                roll REAL,
                lap_time REAL,
                lap_distance REAL,
                PRIMARY KEY (session_id, frame_identifier, car_index)
            )",
            [],
        )?;
        
        self.conn = Some(conn);
        
        Ok(())
    }

    pub(crate) fn write_frame_to_db(&mut self, session_id: u64, session_time: f32, frame_id: u32) -> SqlResult<()> {
        if self.conn.is_none() {
            return Ok(());
        }
        
        if let Some(frame_data) = self.frame_cache.remove(&frame_id) {
            let tx = self.conn.as_mut().unwrap().transaction()?;
            
            for (car_index, car_data) in frame_data.iter().enumerate() {
                tx.execute(
                    "INSERT INTO telemetry (
                        session_id, frame_identifier, car_index, session_time, 
                        speed, rpm, throttle, steer, brake, clutch, gear, drs,
                        brake_temperature_front_left, brake_temperature_front_right, brake_temperature_rear_left, brake_temperature_rear_right,
                        tyre_surface_temperature_front_left, tyre_surface_temperature_front_right, tyre_surface_temperature_rear_left, tyre_surface_temperature_rear_right,
                        tyre_carcass_temperature_front_left, tyre_carcass_temperature_front_right, tyre_carcass_temperature_rear_left, tyre_carcass_temperature_rear_right,
                        tyre_pressure_front_left, tyre_pressure_front_right, tyre_pressure_rear_left, tyre_pressure_rear_right,
                        engine_temperature,
                        world_position_x, world_position_y, world_position_z,
                        g_force_lateral, g_force_longitudinal, g_force_vertical,
                        yaw, pitch, roll,
                        lap_time, lap_distance
                    ) VALUES (
                        :session_id, :frame_id, :car_index, :session_time,
                        :speed, :rpm, :throttle, :steer, :brake, :clutch, :gear, :drs,
                        :brake_temperature_front_left, :brake_temperature_front_right, :brake_temperature_rear_left, :brake_temperature_rear_right,
                        :tyre_surface_temperature_front_left, :tyre_surface_temperature_front_right, :tyre_surface_temperature_rear_left, :tyre_surface_temperature_rear_right,
                        :tyre_carcass_temperature_front_left, :tyre_carcass_temperature_front_right, :tyre_carcass_temperature_rear_left, :tyre_carcass_temperature_rear_right,
                        :tyre_pressure_front_left, :tyre_pressure_front_right, :tyre_pressure_rear_left, :tyre_pressure_rear_right,
                        :engine_temperature,
                        :world_position_x, :world_position_y, :world_position_z,
                        :g_force_lateral, :g_force_longitudinal, :g_force_vertical,
                        :yaw, :pitch, :roll,
                        :lap_time, :lap_distance
                    )",
                    rusqlite::named_params! {
                        ":session_id": session_id.to_string(),
                        ":frame_id": frame_id,
                        ":car_index": car_index as u32,
                        ":session_time": session_time,
                        ":speed": car_data.speed,
                        ":rpm": car_data.rpm,
                        ":throttle": car_data.throttle,
                        ":steer": car_data.steer,
                        ":brake": car_data.brake,
                        ":clutch": car_data.clutch,
                        ":gear": car_data.gear,
                        ":drs": car_data.drs,
                        ":brake_temperature_front_left": car_data.brake_temperature_front_left,
                        ":brake_temperature_front_right": car_data.brake_temperature_front_right,
                        ":brake_temperature_rear_left": car_data.brake_temperature_rear_left,
                        ":brake_temperature_rear_right": car_data.brake_temperature_rear_right,
                        ":tyre_surface_temperature_front_left": car_data.tyre_surface_temperature_front_left,
                        ":tyre_surface_temperature_front_right": car_data.tyre_surface_temperature_front_right,
                        ":tyre_surface_temperature_rear_left": car_data.tyre_surface_temperature_rear_left,
                        ":tyre_surface_temperature_rear_right": car_data.tyre_surface_temperature_rear_right,
                        ":tyre_carcass_temperature_front_left": car_data.tyre_carcass_temperature_front_left,
                        ":tyre_carcass_temperature_front_right": car_data.tyre_carcass_temperature_front_right,
                        ":tyre_carcass_temperature_rear_left": car_data.tyre_carcass_temperature_rear_left,
                        ":tyre_carcass_temperature_rear_right": car_data.tyre_carcass_temperature_rear_right,
                        ":tyre_pressure_front_left": car_data.tyre_pressure_front_left,
                        ":tyre_pressure_front_right": car_data.tyre_pressure_front_right,
                        ":tyre_pressure_rear_left": car_data.tyre_pressure_rear_left,
                        ":tyre_pressure_rear_right": car_data.tyre_pressure_rear_right,
                        ":engine_temperature": car_data.engine_temperature,
                        ":world_position_x": car_data.world_position_x,
                        ":world_position_y": car_data.world_position_y,
                        ":world_position_z": car_data.world_position_z,
                        ":g_force_lateral": car_data.g_force_lateral,
                        ":g_force_longitudinal": car_data.g_force_longitudinal,
                        ":g_force_vertical": car_data.g_force_vertical,
                        ":yaw": car_data.yaw,
                        ":pitch": car_data.pitch,
                        ":roll": car_data.roll,
                        ":lap_time": car_data.lap_time,
                        ":lap_distance": car_data.lap_distance,
                    },
                )?;
            }
            
            tx.commit()?;
        }
        
        Ok(())
    }

    pub fn update_frame_with_telemetry(&mut self, frame_id: u32, car_data: Vec<CarTelemetry>) -> SqlResult<()> {
        // Get existing frame data or create new
        let frame_data = self.frame_cache.entry(frame_id).or_insert_with(|| {
            vec![CarTelemetry::default(); car_data.len()]
        });

        // If the cached frame has fewer cars than the incoming data, resize it
        if frame_data.len() < car_data.len() {
            frame_data.resize(car_data.len(), CarTelemetry::default());
        }

        // Merge the new data with existing data
        for (i, new_car_data) in car_data.into_iter().enumerate() {
            let existing_car = &mut frame_data[i];
            
            // Only update fields that are present in the new data
            if new_car_data.speed.is_some() { existing_car.speed = new_car_data.speed; }
            if new_car_data.rpm.is_some() { existing_car.rpm = new_car_data.rpm; }
            if new_car_data.throttle.is_some() { existing_car.throttle = new_car_data.throttle; }
            if new_car_data.steer.is_some() { existing_car.steer = new_car_data.steer; }
            if new_car_data.brake.is_some() { existing_car.brake = new_car_data.brake; }
            if new_car_data.clutch.is_some() { existing_car.clutch = new_car_data.clutch; }
            if new_car_data.gear.is_some() { existing_car.gear = new_car_data.gear; }
            if new_car_data.drs.is_some() { existing_car.drs = new_car_data.drs; }
            
            if new_car_data.brake_temperature_front_left.is_some() { existing_car.brake_temperature_front_left = new_car_data.brake_temperature_front_left; }
            if new_car_data.brake_temperature_front_right.is_some() { existing_car.brake_temperature_front_right = new_car_data.brake_temperature_front_right; }
            if new_car_data.brake_temperature_rear_left.is_some() { existing_car.brake_temperature_rear_left = new_car_data.brake_temperature_rear_left; }
            if new_car_data.brake_temperature_rear_right.is_some() { existing_car.brake_temperature_rear_right = new_car_data.brake_temperature_rear_right; }
            
            if new_car_data.tyre_surface_temperature_front_left.is_some() { existing_car.tyre_surface_temperature_front_left = new_car_data.tyre_surface_temperature_front_left; }
            if new_car_data.tyre_surface_temperature_front_right.is_some() { existing_car.tyre_surface_temperature_front_right = new_car_data.tyre_surface_temperature_front_right; }
            if new_car_data.tyre_surface_temperature_rear_left.is_some() { existing_car.tyre_surface_temperature_rear_left = new_car_data.tyre_surface_temperature_rear_left; }
            if new_car_data.tyre_surface_temperature_rear_right.is_some() { existing_car.tyre_surface_temperature_rear_right = new_car_data.tyre_surface_temperature_rear_right; }
            
            if new_car_data.tyre_carcass_temperature_front_left.is_some() { existing_car.tyre_carcass_temperature_front_left = new_car_data.tyre_carcass_temperature_front_left; }
            if new_car_data.tyre_carcass_temperature_front_right.is_some() { existing_car.tyre_carcass_temperature_front_right = new_car_data.tyre_carcass_temperature_front_right; }
            if new_car_data.tyre_carcass_temperature_rear_left.is_some() { existing_car.tyre_carcass_temperature_rear_left = new_car_data.tyre_carcass_temperature_rear_left; }
            if new_car_data.tyre_carcass_temperature_rear_right.is_some() { existing_car.tyre_carcass_temperature_rear_right = new_car_data.tyre_carcass_temperature_rear_right; }
            
            if new_car_data.tyre_pressure_front_left.is_some() { existing_car.tyre_pressure_front_left = new_car_data.tyre_pressure_front_left; }
            if new_car_data.tyre_pressure_front_right.is_some() { existing_car.tyre_pressure_front_right = new_car_data.tyre_pressure_front_right; }
            if new_car_data.tyre_pressure_rear_left.is_some() { existing_car.tyre_pressure_rear_left = new_car_data.tyre_pressure_rear_left; }
            if new_car_data.tyre_pressure_rear_right.is_some() { existing_car.tyre_pressure_rear_right = new_car_data.tyre_pressure_rear_right; }
            
            if new_car_data.engine_temperature.is_some() { existing_car.engine_temperature = new_car_data.engine_temperature; }
            
            if new_car_data.world_position_x.is_some() { existing_car.world_position_x = new_car_data.world_position_x; }
            if new_car_data.world_position_y.is_some() { existing_car.world_position_y = new_car_data.world_position_y; }
            if new_car_data.world_position_z.is_some() { existing_car.world_position_z = new_car_data.world_position_z; }
            
            if new_car_data.g_force_lateral.is_some() { existing_car.g_force_lateral = new_car_data.g_force_lateral; }
            if new_car_data.g_force_longitudinal.is_some() { existing_car.g_force_longitudinal = new_car_data.g_force_longitudinal; }
            if new_car_data.g_force_vertical.is_some() { existing_car.g_force_vertical = new_car_data.g_force_vertical; }
            
            if new_car_data.yaw.is_some() { existing_car.yaw = new_car_data.yaw; }
            if new_car_data.pitch.is_some() { existing_car.pitch = new_car_data.pitch; }
            if new_car_data.roll.is_some() { existing_car.roll = new_car_data.roll; }
            
            if new_car_data.lap_time.is_some() { existing_car.lap_time = new_car_data.lap_time; }
            if new_car_data.lap_distance.is_some() { existing_car.lap_distance = new_car_data.lap_distance; }
        }
        
        Ok(())
    }

    pub fn handle_new_frame(&mut self, session_id: u64, session_time: f32, frame_id: u32) -> SqlResult<()> {
        // Ignore packets with session_id == 0
        if session_id == 0 {
            return Ok(());
        }
        
        // If no database connection yet, just cache the frame metadata
        if !self.has_connection() {
            self.current_frame = Some(frame_id);
            self.session_id = Some(session_id);
            self.session_time = Some(session_time);
            return Ok(());
        }

        if let Some(current_frame) = self.current_frame {
            if current_frame != frame_id {
                if let (Some(sid), Some(st)) = (self.session_id, self.session_time) {
                    self.write_frame_to_db(sid, st, current_frame)?;
                }
            }
        }
        
        self.current_frame = Some(frame_id);
        self.session_id = Some(session_id);
        self.session_time = Some(session_time);
        Ok(())
    }

    pub fn flush_and_close(&mut self) -> SqlResult<()> {
        if self.conn.is_none() {
            return Ok(());
        }

        // First write any pending frame
        if let (Some(frame_id), Some(sid), Some(st)) = (self.current_frame, self.session_id, self.session_time) {
            self.write_frame_to_db(sid, st, frame_id)?;
        }

        // Force a full checkpoint and switch back to DELETE mode to cleanup WAL files
        let conn = self.conn.as_mut().unwrap();
        conn.pragma_update(None, "wal_checkpoint", &"TRUNCATE")?;
        conn.pragma_update(None, "journal_mode", &"DELETE")?;
        
        Ok(())
    }
}

impl Drop for DbHandler {
    fn drop(&mut self) {
        if self.conn.is_none() {
            return;
        }
        
        // First try to write any pending frame
        if let (Some(frame_id), Some(sid), Some(st)) = (self.current_frame, self.session_id, self.session_time) {
            if let Err(e) = self.write_frame_to_db(sid, st, frame_id) {
                eprintln!("Error writing final frame to database: {}", e);
            }
        }

        // Then try to properly close the WAL mode
        let conn = self.conn.as_mut().unwrap();
        if let Err(e) = conn.pragma_update(None, "wal_checkpoint", &"TRUNCATE") {
            eprintln!("Error checkpointing WAL: {}", e);
        }
        if let Err(e) = conn.pragma_update(None, "journal_mode", &"DELETE") {
            eprintln!("Error switching journal mode: {}", e);
        }
    }
} 