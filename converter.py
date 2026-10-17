import zstandard as zstd
import json
import pandas as pd
from pathlib import Path
import argparse
from typing import Dict

PACKET_TYPES = {
    0: "CarMotion",
    1: "Session",
    2: "Lap",
    3: "Event",
    4: "Participants",
    5: "CarSetup",
    6: "CarTelemetry",
    7: "CarStatus",
    8: "FinalClassification",
    9: "LobbyInfo",
    10: "CarDamage",
    11: "SessionHistory",
    12: "TyreSets",
    13: "CarMotionEx",
    14: "TimeTrial",
}


class TelemetryConverter:
    def __init__(self, input_file: str, output_dir: str, partition_size: int = 100000):
        self.input_file = Path(input_file)
        self.output_dir = Path(output_dir)
        self.output_dir.mkdir(exist_ok=True, parents=True)
        self.partition_size = partition_size

        self.packet_handlers = {
            0: self._handle_car_motion,
            1: self._handle_session,
            2: self._handle_lap,
            3: self._handle_event,
            4: self._handle_participants,
            5: self._handle_car_setup,
            6: self._handle_car_telemetry,
            7: self._handle_car_status,
            8: self._handle_final_classification,
            9: self._handle_lobby_info,
            10: self._handle_car_damage,
            11: self._handle_session_history,
            12: self._handle_tyre_sets,
            13: self._handle_car_motion_ex,
            14: self._handle_time_trial,
        }

        # Data buffers for each packet type (list of flattened packets)
        self.data_buffers = {packet_id: [] for packet_id in PACKET_TYPES}

        # Track counts for each packet type
        self.packet_counts = {packet_id: 0 for packet_id in PACKET_TYPES}

    def read_json(self) -> Dict:
        """Read the input JSON file (possibly compressed)"""
        print(f"Reading {self.input_file}...")

        if self.input_file.suffix == ".zst":
            with open(self.input_file, "rb") as f:
                decompressor = zstd.ZstdDecompressor()
                with decompressor.stream_reader(f) as reader:
                    data = json.load(reader)
        else:
            with open(self.input_file, "r") as f:
                data = json.load(f)

        print(f"Found {len(data['packets'])} packets")
        return data

    def process_packets(self, data: Dict):
        """Process all packets in the telemetry data"""
        metadata = data["metadata"]
        session_id = metadata["session_id"]
        timestamp = metadata["timestamp"]

        # Process each packet
        for i, packet in enumerate(data["packets"]):
            packet_type = packet["packet_type"]

            # Skip unknown packet types
            if packet_type not in PACKET_TYPES:
                continue

            base_data = {
                "session_id": session_id,
                "file_timestamp": timestamp,
                "packet_timestamp_us": packet["timestamp_us"],
            }

            packet_name = PACKET_TYPES.get(packet["packet_type"], "")
            packet_data = packet["data"][packet_name]

            # Process packet based on its type
            if packet_type in self.packet_handlers:
                self.packet_handlers[packet_type](packet_data, base_data)

            # Write partition to disk if buffer gets too large
            if (
                self.packet_counts[packet_type] > 0
                and self.packet_counts[packet_type] % self.partition_size == 0
            ):
                self._write_buffer_to_parquet(packet_type)

            # Print progress
            if (i + 1) % 10000 == 0:
                print(f"Processed {i+1}/{len(data['packets'])} packets")

    def _handle_car_telemetry(self, packet_data: Dict, base_data: Dict):
        """Handle CarTelemetry packets"""
        header = packet_data["m_header"]

        # Add header fields
        for key, value in header.items():
            base_data[f"header_{key}"] = value

        # Process car-specific data
        car_data = packet_data["m_carTelemetryData"]
        for car_idx, car in enumerate(car_data):
            # Create a copy of base data for this car
            car_record = base_data.copy()
            car_record["car_idx"] = car_idx

            # Add car telemetry fields
            for key, value in car.items():
                # Handle arrays separately
                if isinstance(value, list):
                    for i, v in enumerate(value):
                        car_record[f"telemetry_{key}_{i}"] = v
                else:
                    car_record[f"telemetry_{key}"] = value

            # Add additional fields
            if "m_mfdPanel" in packet_data:
                car_record["telemetry_mfd_panel"] = packet_data["m_mfdPanel"]
            if "m_mfdPanelSecondaryPlayer" in packet_data:
                car_record["telemetry_mfd_panel_secondary"] = packet_data[
                    "m_mfdPanelSecondaryPlayer"
                ]
            if "m_suggestedGear" in packet_data:
                car_record["telemetry_suggested_gear"] = packet_data["m_suggestedGear"]

            # Add to buffer
            self.data_buffers[6].append(car_record)
            self.packet_counts[6] += 1

    # Add similar handlers for other packet types
    def _handle_car_motion(self, packet_data: Dict, base_data: Dict):
        """Handle CarMotion packets"""
        header = packet_data["m_header"]

        # Add header fields
        for key, value in header.items():
            base_data[f"header_{key}"] = value

        # Process car-specific motion data
        car_data = packet_data["m_carMotionData"]
        for car_idx, car in enumerate(car_data):
            # Create a copy of base data for this car
            car_record = base_data.copy()
            car_record["car_idx"] = car_idx

            # Add car motion fields
            for key, value in car.items():
                car_record[f"car_{key}"] = value

            # Add additional fields that might be present at packet level
            if "m_suspensionPosition" in packet_data:
                car_record["suspension_position"] = (
                    packet_data["m_suspensionPosition"][car_idx]
                    if car_idx < len(packet_data["m_suspensionPosition"])
                    else None
                )
            if "m_suspensionVelocity" in packet_data:
                car_record["suspension_velocity"] = (
                    packet_data["m_suspensionVelocity"][car_idx]
                    if car_idx < len(packet_data["m_suspensionVelocity"])
                    else None
                )
            if "m_suspensionAcceleration" in packet_data:
                car_record["suspension_acceleration"] = (
                    packet_data["m_suspensionAcceleration"][car_idx]
                    if car_idx < len(packet_data["m_suspensionAcceleration"])
                    else None
                )
            if "m_wheelSpeed" in packet_data:
                car_record["wheel_speed"] = (
                    packet_data["m_wheelSpeed"][car_idx]
                    if car_idx < len(packet_data["m_wheelSpeed"])
                    else None
                )
            if "m_wheelSlip" in packet_data:
                car_record["wheel_slip"] = (
                    packet_data["m_wheelSlip"][car_idx]
                    if car_idx < len(packet_data["m_wheelSlip"])
                    else None
                )
            if "m_localVelocityX" in packet_data:
                car_record["local_velocity_x"] = (
                    packet_data["m_localVelocityX"][car_idx]
                    if car_idx < len(packet_data["m_localVelocityX"])
                    else None
                )
            if "m_localVelocityY" in packet_data:
                car_record["local_velocity_y"] = (
                    packet_data["m_localVelocityY"][car_idx]
                    if car_idx < len(packet_data["m_localVelocityY"])
                    else None
                )
            if "m_localVelocityZ" in packet_data:
                car_record["local_velocity_z"] = (
                    packet_data["m_localVelocityZ"][car_idx]
                    if car_idx < len(packet_data["m_localVelocityZ"])
                    else None
                )

            # Add to buffer
            self.data_buffers[0].append(car_record)
            self.packet_counts[0] += 1

    def _handle_car_motion_ex(self, packet_data: Dict, base_data: Dict):
        """Handle CarMotionEx packets (extended motion data)"""
        header = packet_data["m_header"]

        # Add header fields
        for key, value in header.items():
            base_data[f"header_{key}"] = value

        # This is usually player car specific data
        car_record = base_data.copy()
        car_record["car_idx"] = header["m_playerCarIndex"]

        # Add the extended motion data fields
        for key, value in packet_data.items():
            if key != "header":
                car_record[f"ex_{key}"] = value

        # Add to buffer
        self.data_buffers[13].append(car_record)
        self.packet_counts[13] += 1

    def _handle_session(self, packet_data: Dict, base_data: Dict):
        """Handle Session packets"""
        header = packet_data["m_header"]

        # Add header fields
        for key, value in header.items():
            base_data[f"header_{key}"] = value

        # Session data is global, not car-specific
        session_record = base_data.copy()

        # Add all session fields
        for key, value in packet_data.items():
            if key != "header":
                # Handle marshal zones array
                if key == "m_marshalZones":
                    for i, zone in enumerate(value):
                        for zone_key, zone_value in zone.items():
                            session_record[f"marshal_zone_{i}_{zone_key}"] = zone_value
                # Handle weather forecast array
                elif key == "m_weatherForecastSamples":
                    for i, forecast in enumerate(value):
                        for forecast_key, forecast_value in forecast.items():
                            session_record[f"weather_forecast_{i}_{forecast_key}"] = (
                                forecast_value
                            )
                # Normal fields
                else:
                    session_record[f"session_{key}"] = value

        # Add to buffer
        self.data_buffers[1].append(session_record)
        self.packet_counts[1] += 1

    def _handle_lap(self, packet_data: Dict, base_data: Dict):
        """Handle Lap Data packets"""
        header = packet_data["m_header"]

        # Add header fields
        for key, value in header.items():
            base_data[f"header_{key}"] = value

        # Process car-specific lap data
        lap_data = packet_data["m_lapData"]
        for car_idx, car in enumerate(lap_data):
            # Create a copy of base data for this car
            car_record = base_data.copy()
            car_record["car_idx"] = car_idx

            # Add lap data fields
            for key, value in car.items():
                car_record[f"lap_{key}"] = value

            # Add additional fields
            if "m_timeTrialPBCarIdx" in packet_data:
                car_record["time_trial_pb_car_idx"] = packet_data["m_timeTrialPBCarIdx"]
            if "m_timeTrialRivalCarIdx" in packet_data:
                car_record["time_trial_rival_car_idx"] = packet_data["m_timeTrialRivalCarIdx"]

            # Add to buffer
            self.data_buffers[2].append(car_record)
            self.packet_counts[2] += 1

    def _handle_event(self, packet_data: Dict, base_data: Dict):
        """Handle Event packets"""
        header = packet_data["m_header"]

        # Add header fields
        for key, value in header.items():
            base_data[f"header_{key}"] = value

        # Create event record
        event_record = base_data.copy()
        event_record["event_code"] = packet_data["m_eventStringCode"]

        # Extract event details
        event_details = packet_data["m_eventDetails"]

        # Handle different event types
        if isinstance(event_details, dict):
            event_type = event_details.get("type", "Unknown")
            event_record["event_type"] = event_type

            # Add all event details fields
            for key, value in event_details.items():
                if key != "type":
                    event_record[f"event_{key}"] = value

        # Add to buffer
        self.data_buffers[3].append(event_record)
        self.packet_counts[3] += 1

    def _handle_participants(self, packet_data: Dict, base_data: Dict):
        """Handle Participants packets"""
        header = packet_data["m_header"]

        # Add header fields
        for key, value in header.items():
            base_data[f"header_{key}"] = value

        # Process participant data
        participants = packet_data["m_participants"]
        for participant_idx, participant in enumerate(participants):
            # Only process up to m_numActiveCars
            if participant_idx >= packet_data.get("m_numActiveCars", len(participants)):
                break

            # Create a copy of base data for this participant
            participant_record = base_data.copy()
            participant_record["participant_idx"] = participant_idx

            # Add participant fields
            for key, value in participant.items():
                participant_record[f"participant_{key}"] = value

            # Add to buffer
            self.data_buffers[4].append(participant_record)
            self.packet_counts[4] += 1

    def _handle_car_setup(self, packet_data: Dict, base_data: Dict):
        """Handle Car Setup packets"""
        header = packet_data["m_header"]

        # Add header fields
        for key, value in header.items():
            base_data[f"header_{key}"] = value

        # Process car-specific setup data
        car_setups = packet_data["m_carSetups"]
        for car_idx, setup in enumerate(car_setups):
            # Create a copy of base data for this car
            car_record = base_data.copy()
            car_record["car_idx"] = car_idx

            # Add setup fields
            for key, value in setup.items():
                car_record[f"setup_{key}"] = value

            # Add to buffer
            self.data_buffers[5].append(car_record)
            self.packet_counts[5] += 1

    def _handle_car_status(self, packet_data: Dict, base_data: Dict):
        """Handle Car Status packets"""
        header = packet_data["m_header"]

        # Add header fields
        for key, value in header.items():
            base_data[f"header_{key}"] = value

        # Process car-specific status data
        car_status_data = packet_data["m_carStatusData"]
        for car_idx, status in enumerate(car_status_data):
            # Create a copy of base data for this car
            car_record = base_data.copy()
            car_record["car_idx"] = car_idx

            # Add status fields
            for key, value in status.items():
                car_record[f"status_{key}"] = value

            # Add to buffer
            self.data_buffers[7].append(car_record)
            self.packet_counts[7] += 1

    def _handle_car_damage(self, packet_data: Dict, base_data: Dict):
        """Handle Car Damage packets"""
        header = packet_data["m_header"]

        # Add header fields
        for key, value in header.items():
            base_data[f"header_{key}"] = value

        # Process car-specific damage data
        car_damage_data = packet_data["m_carDamageData"]
        for car_idx, damage in enumerate(car_damage_data):
            # Create a copy of base data for this car
            car_record = base_data.copy()
            car_record["car_idx"] = car_idx

            # Add damage fields
            for key, value in damage.items():
                # Handle arrays (tyres, brakes, etc.)
                if isinstance(value, list):
                    for i, v in enumerate(value):
                        car_record[f"damage_{key}_{i}"] = v
                else:
                    car_record[f"damage_{key}"] = value

            # Add to buffer
            self.data_buffers[10].append(car_record)
            self.packet_counts[10] += 1

    def _handle_final_classification(self, packet_data: Dict, base_data: Dict):
        """Handle Final Classification packets"""
        header = packet_data["m_header"]

        # Add header fields
        for key, value in header.items():
            base_data[f"header_{key}"] = value

        # Process classification data for each car
        classification_data = packet_data["m_classificationData"]
        for car_idx, classification in enumerate(classification_data):
            # Only process up to m_numCars
            if car_idx >= packet_data.get("m_numCars", len(classification_data)):
                break

            # Create a copy of base data for this car
            car_record = base_data.copy()
            car_record["car_idx"] = car_idx

            # Add classification fields
            for key, value in classification.items():
                # Handle arrays (tyre stints)
                if isinstance(value, list):
                    for i, v in enumerate(value):
                        car_record[f"classification_{key}_{i}"] = v
                else:
                    car_record[f"classification_{key}"] = value

            # Add to buffer
            self.data_buffers[8].append(car_record)
            self.packet_counts[8] += 1

    def _handle_lobby_info(self, packet_data: Dict, base_data: Dict):
        """Handle Lobby Info packets"""
        header = packet_data["m_header"]

        # Add header fields
        for key, value in header.items():
            base_data[f"header_{key}"] = value

        # Process lobby player data
        lobby_players = packet_data["m_lobbyPlayers"]
        for player_idx, player in enumerate(lobby_players):
            # Only process up to m_numPlayers
            if player_idx >= packet_data.get("m_numPlayers", len(lobby_players)):
                break

            # Create a copy of base data for this player
            player_record = base_data.copy()
            player_record["player_idx"] = player_idx

            # Add lobby player fields
            for key, value in player.items():
                player_record[f"lobby_{key}"] = value

            # Add to buffer
            self.data_buffers[9].append(player_record)
            self.packet_counts[9] += 1

    def _handle_session_history(self, packet_data: Dict, base_data: Dict):
        """Handle Session History packets"""
        header = packet_data["m_header"]

        # Add header fields
        for key, value in header.items():
            base_data[f"header_{key}"] = value

        # Create record
        history_record = base_data.copy()
        history_record["car_idx"] = packet_data.get("m_carIdx", 0)

        # Add basic fields
        for key, value in packet_data.items():
            if key not in ["header", "m_lapHistoryData", "m_tyreStintsHistoryData"]:
                history_record[f"history_{key}"] = value

        # Process lap history data
        if "m_lapHistoryData" in packet_data:
            lap_history = packet_data["m_lapHistoryData"]
            for lap_idx, lap in enumerate(lap_history):
                # Only process up to m_numLaps
                if lap_idx >= packet_data.get("m_numLaps", len(lap_history)):
                    break

                # Create a new record for each lap
                lap_record = history_record.copy()
                lap_record["lap_idx"] = lap_idx

                # Add lap data
                for key, value in lap.items():
                    lap_record[f"lap_{key}"] = value

                # Add to buffer
                self.data_buffers[11].append(lap_record)
                self.packet_counts[11] += 1

        # Process tyre stints history (optional, could use a separate buffer)
        if "m_tyreStintsHistoryData" in packet_data and not packet_data.get(
            "m_lapHistoryData"
        ):
            # If we didn't process lap history, add a base record
            self.data_buffers[11].append(history_record)
            self.packet_counts[11] += 1

    def _handle_tyre_sets(self, packet_data: Dict, base_data: Dict):
        """Handle Tyre Sets packets"""
        header = packet_data["m_header"]

        # Add header fields
        for key, value in header.items():
            base_data[f"header_{key}"] = value

        # Create base record
        tyre_record = base_data.copy()
        tyre_record["car_idx"] = packet_data.get("m_carIdx", 0)

        # Add fitted index
        if "m_fittedIdx" in packet_data:
            tyre_record["fitted_idx"] = packet_data["m_fittedIdx"]

        # Process tyre set data
        if "m_tyreSetData" in packet_data:
            tyre_sets = packet_data["m_tyreSetData"]
            for set_idx, tyre_set in enumerate(tyre_sets):
                # Create a new record for each tyre set
                set_record = tyre_record.copy()
                set_record["tyre_set_idx"] = set_idx

                # Add tyre set data
                for key, value in tyre_set.items():
                    set_record[f"tyre_set_{key}"] = value

                # Add to buffer
                self.data_buffers[12].append(set_record)
                self.packet_counts[12] += 1
        else:
            # If no tyre set data, just add the base record
            self.data_buffers[12].append(tyre_record)
            self.packet_counts[12] += 1

    def _handle_time_trial(self, packet_data: Dict, base_data: Dict):
        """Handle Time Trial packets"""
        header = packet_data["m_header"]

        # Add header fields
        for key, value in header.items():
            base_data[f"header_{key}"] = value

        # Time trial data is not car-specific
        time_trial_record = base_data.copy()

        # Add all time trial fields
        for key, value in packet_data.items():
            if key != "header":
                time_trial_record[f"time_trial_{key}"] = value

        # Add to buffer
        self.data_buffers[14].append(time_trial_record)
        self.packet_counts[14] += 1

    def _write_buffer_to_parquet(self, packet_type: int):
        """Write a buffer to a Parquet file"""
        buffer = self.data_buffers[packet_type]
        if not buffer:
            return

        packet_name = PACKET_TYPES[packet_type]
        count = self.packet_counts[packet_type]
        partition = count // self.partition_size

        # Convert to DataFrame
        df = pd.DataFrame(buffer)

        # Save to Parquet
        output_file = self.output_dir / f"{packet_name}_part{partition}.parquet"
        df.to_parquet(output_file, engine="pyarrow", compression="snappy")

        print(f"Wrote {len(buffer)} {packet_name} records to {output_file}")

        # Clear buffer
        self.data_buffers[packet_type] = []

    def finalize(self):
        """Write any remaining data to disk"""
        for packet_type in PACKET_TYPES:
            if self.data_buffers[packet_type]:
                self._write_buffer_to_parquet(packet_type)


def main():
    parser = argparse.ArgumentParser(
        description="Convert F1 telemetry data to parquet files"
    )
    parser.add_argument("input", help="Input JSON or JSON.zst file")
    parser.add_argument(
        "--output", "-o", default="telemetry_data", help="Output directory"
    )
    parser.add_argument(
        "--partition-size",
        "-p",
        type=int,
        default=100000,
        help="Number of packets per partition file",
    )
    args = parser.parse_args()

    converter = TelemetryConverter(args.input, args.output, args.partition_size)
    data = converter.read_json()
    converter.process_packets(data)
    converter.finalize()

    print("Conversion complete!")


if __name__ == "__main__":
    main()
