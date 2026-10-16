import sqlite3
import sys


def analyse_telemetry(db_path, car_index=0):
    conn = sqlite3.connect(db_path)
    cursor = conn.cursor()

    # Get frame range and count
    cursor.execute(
        """
        SELECT 
            MIN(frame_identifier) as min_frame,
            MAX(frame_identifier) as max_frame,
            COUNT(*) as total_frames,
            MIN(session_time) as start_time,
            MAX(session_time) as end_time
        FROM telemetry 
        WHERE car_index = ?
    """,
        (car_index,),
    )

    stats = cursor.fetchone()
    min_frame, max_frame, total_frames, start_time, end_time = stats

    print(f"\nAnalysis for car {car_index}:")
    print(f"Frame range: {min_frame} to {max_frame}")
    print(f"Total frames captured: {total_frames}")
    print(f"Session time range: {start_time:.2f}s to {end_time:.2f}s")

    expected_frames = max_frame - min_frame + 1
    dropped_frames = expected_frames - total_frames

    print(f"\nExpected frames: {expected_frames}")
    print(f"Dropped frames: {dropped_frames}")
    print(f"Packet capture rate: {(total_frames/expected_frames)*100:.2f}%")

    # Find gaps in frame sequence
    cursor.execute(
        """
        WITH RECURSIVE
        sequence(frame_num) AS (
            SELECT MIN(frame_identifier) FROM telemetry WHERE car_index = ?
            UNION ALL
            SELECT frame_num + 1
            FROM sequence
            WHERE frame_num < (SELECT MAX(frame_identifier) FROM telemetry WHERE car_index = ?)
        )
        SELECT s.frame_num
        FROM sequence s
        LEFT JOIN telemetry t ON t.frame_identifier = s.frame_num AND t.car_index = ?
        WHERE t.frame_identifier IS NULL
        LIMIT 10
    """,
        (car_index, car_index, car_index),
    )

    gaps = cursor.fetchall()
    if gaps:
        print("\nFirst 10 missing frame_identifiers:")
        for gap in gaps:
            print(f"Missing frame: {gap[0]}")

    conn.close()


if __name__ == "__main__":
    db_path = "./aggregator/telemetry.db"
    car_index = 0  # Default to first car

    if len(sys.argv) > 1:
        car_index = int(sys.argv[1])

    analyse_telemetry(db_path, car_index)
