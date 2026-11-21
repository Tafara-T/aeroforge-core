import sqlite3
import json
import hashlib
from datetime import datetime, timezone

class TamperEvidentArchive:
    def __init__(self, db_path: str = "af_evidence.db"):
        self.db_path = db_path
        self._init_db()
        self._last_hash = self._get_last_hash()

    def _init_db(self):
        with sqlite3.connect(self.db_path) as conn:
            conn.execute("""
                CREATE TABLE IF NOT EXISTS state_updates (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    record_hash TEXT NOT NULL UNIQUE,
                    prev_hash TEXT NOT NULL,
                    entity_id TEXT NOT NULL,
                    variable_name TEXT NOT NULL,
                    value_json TEXT NOT NULL,
                    timestamp_observed TEXT NOT NULL,
                    timestamp_ingested TEXT NOT NULL,
                    source_id TEXT NOT NULL,
                    provenance_hash TEXT NOT NULL,
                    uncertainty_state TEXT NOT NULL,
                    created_at TEXT NOT NULL
                )
            """)
            conn.commit()

    def _compute_record_hash(self, record_data, prev_hash):
        canonical = json.dumps({
            "prev_hash": prev_hash,
            "entity_id": record_data["entity_id"],
            "variable_name": record_data["variable_name"],
            "value_json": record_data["value_json"],
            "timestamp_observed": record_data["timestamp_observed"],
            "source_id": record_data["source_id"],
            "provenance_hash": record_data["provenance_hash"],
            "uncertainty_state": record_data["uncertainty_state"],
        }, sort_keys=True, separators=(',', ':'))
        return hashlib.sha256(canonical.encode()).hexdigest()

    def _get_last_hash(self):
        with sqlite3.connect(self.db_path) as conn:
            cursor = conn.execute(
                "SELECT record_hash FROM state_updates ORDER BY id DESC LIMIT 1"
            )
            row = cursor.fetchone()
            return row[0] if row else "0" * 64

    def append_update(self, update):
        value_json = json.dumps(update["value"], sort_keys=True)
        record_data = {
            "entity_id": update["entity_id"],
            "variable_name": update["variable_name"],
            "value_json": value_json,
            "timestamp_observed": update["temporal"]["timestamp_observed"],
            "timestamp_ingested": update["temporal"]["timestamp_ingested"],
            "source_id": update["provenance"]["source_id"],
            "provenance_hash": update["provenance"]["provenance_hash"],
            "uncertainty_state": json.dumps(update["uncertainty"], sort_keys=True),
        }
        record_hash = self._compute_record_hash(record_data, self._last_hash)
        created_at = datetime.now(timezone.utc).isoformat()

        with sqlite3.connect(self.db_path) as conn:
            conn.execute("""
                INSERT INTO state_updates (
                    record_hash, prev_hash, entity_id, variable_name, value_json,
                    timestamp_observed, timestamp_ingested, source_id,
                    provenance_hash, uncertainty_state, created_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            """, (
                record_hash, self._last_hash,
                record_data["entity_id"], record_data["variable_name"],
                record_data["value_json"], record_data["timestamp_observed"],
                record_data["timestamp_ingested"], record_data["source_id"],
                record_data["provenance_hash"], record_data["uncertainty_state"],
                created_at,
            ))
            conn.commit()
            self._last_hash = record_hash
            return record_hash

    def verify_integrity(self):
        with sqlite3.connect(self.db_path) as conn:
            cursor = conn.execute(
                "SELECT record_hash, prev_hash, entity_id, variable_name, value_json, "
                "timestamp_observed, source_id, provenance_hash, uncertainty_state "
                "FROM state_updates ORDER BY id ASC"
            )
            expected_prev_hash = "0" * 64
            for row in cursor:
                (record_hash, prev_hash, entity_id, variable_name, value_json,
                 timestamp_observed, source_id, provenance_hash, uncertainty_state) = row
                if prev_hash != expected_prev_hash:
                    return False
                record_data = {
                    "entity_id": entity_id,
                    "variable_name": variable_name,
                    "value_json": value_json,
                    "timestamp_observed": timestamp_observed,
                    "source_id": source_id,
                    "provenance_hash": provenance_hash,
                    "uncertainty_state": uncertainty_state,
                }
                expected_hash = self._compute_record_hash(record_data, prev_hash)
                if record_hash != expected_hash:
                    return False
                expected_prev_hash = record_hash
            return True

    def count_records(self):
        with sqlite3.connect(self.db_path) as conn:
            cursor = conn.execute("SELECT COUNT(*) FROM state_updates")
            return cursor.fetchone()[0]
