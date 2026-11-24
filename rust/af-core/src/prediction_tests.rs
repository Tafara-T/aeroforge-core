#[cfg(test)]
mod tests {
    use crate::prediction::{
        predict_fuel_endurance, predict_link_reliability, predict_mission_duration,
        FuelState, EngineFlowState, RadioConfig, WindVector,
    };
    use crate::mission_model::{Task, TaskScheduleEntry, TransitSegment};
    use crate::core_types::{GeoPosition, SourceReference, StateValue};
    use chrono::Utc;

    fn dummy_obs() -> Vec<SourceReference> {
        vec![SourceReference {
            source_id: "test".to_string(),
            timestamp_observed: Utc::now(),
            provenance_hash: "test_hash".to_string(),
        }]
    }

    #[test]
    fn test_nominal_fuel_endurance() {
        let tanks = vec![FuelState { tank_id: "t1".into(), contents_lbs: 100.0 }];
        let engines = vec![EngineFlowState { engine_id: "e1".into(), flow_pps: 0.1 }];
        let res = predict_fuel_endurance(&tanks, &engines, 20.0, dummy_obs());
        assert!(matches!(res.predicted_value, StateValue::Float { value, .. } if value > 0.0));
    }

    #[test]
    fn test_degraded_fuel_nan() {
        let tanks = vec![FuelState { tank_id: "t1".into(), contents_lbs: f64::NAN }];
        let engines = vec![EngineFlowState { engine_id: "e1".into(), flow_pps: 0.1 }];
        let res = predict_fuel_endurance(&tanks, &engines, 20.0, dummy_obs());
        assert!(matches!(res.predicted_value, StateValue::Null));
    }

    #[test]
    fn test_zero_fuel_flow() {
        let tanks = vec![FuelState { tank_id: "t1".into(), contents_lbs: 100.0 }];
        let engines = vec![EngineFlowState { engine_id: "e1".into(), flow_pps: 0.0 }];
        let res = predict_fuel_endurance(&tanks, &engines, 20.0, dummy_obs());
        assert!(matches!(res.predicted_value, StateValue::Null));
    }

    #[test]
    fn test_multiple_tanks_and_engines() {
        let tanks = vec![
            FuelState { tank_id: "t1".into(), contents_lbs: 50.0 },
            FuelState { tank_id: "t2".into(), contents_lbs: 50.0 },
        ];
        let engines = vec![
            EngineFlowState { engine_id: "e1".into(), flow_pps: 0.05 },
            EngineFlowState { engine_id: "e2".into(), flow_pps: 0.05 },
        ];
        let res = predict_fuel_endurance(&tanks, &engines, 20.0, dummy_obs());
        assert!(matches!(res.predicted_value, StateValue::Float { value, .. } if value > 0.0));
    }

    #[test]
    fn test_nominal_link_reliability() {
        let ac_pos = GeoPosition { latitude: 45.0, longitude: -122.0, altitude: 1000.0 };
        let gs_pos = GeoPosition { latitude: 45.0, longitude: -122.0, altitude: 500.0 };
        let radio = RadioConfig { 
            freq_mhz: 900.0, tx_power_dbm: 20.0, tx_gain_dbi: 3.0, 
            rx_gain_dbi: 3.0, rx_sensitivity_dbm: -90.0, fade_margin_db: 10.0 
        };
        let res = predict_link_reliability(&ac_pos, &gs_pos, &radio, dummy_obs());
        assert!(matches!(res.predicted_value, StateValue::Float { value, .. } if (0.0..=1.0).contains(&value)));
    }

    #[test]
    fn test_link_terrain_screening() {
        let ac_pos = GeoPosition { latitude: 45.0, longitude: -122.0, altitude: 400.0 }; 
        let gs_pos = GeoPosition { latitude: 45.0, longitude: -122.0, altitude: 500.0 };
        let radio = RadioConfig { 
            freq_mhz: 900.0, tx_power_dbm: 20.0, tx_gain_dbi: 3.0, 
            rx_gain_dbi: 3.0, rx_sensitivity_dbm: -90.0, fade_margin_db: 10.0 
        };
        let res = predict_link_reliability(&ac_pos, &gs_pos, &radio, dummy_obs());
        assert!(matches!(res.predicted_value, StateValue::Null));
    }

    #[test]
    fn test_zero_distance_link() {
        let ac_pos = GeoPosition { latitude: 45.0, longitude: -122.0, altitude: 1000.0 };
        let gs_pos = GeoPosition { latitude: 45.0, longitude: -122.0, altitude: 1000.0 };
        let radio = RadioConfig { 
            freq_mhz: 900.0, tx_power_dbm: 20.0, tx_gain_dbi: 3.0, 
            rx_gain_dbi: 3.0, rx_sensitivity_dbm: -90.0, fade_margin_db: 10.0 
        };
        let res = predict_link_reliability(&ac_pos, &gs_pos, &radio, dummy_obs());
        assert!(matches!(res.predicted_value, StateValue::Null));
    }

    #[test]
    fn test_nominal_mission_duration() {
        let tasks: Vec<Task> = vec![];
        let transits = vec![TransitSegment {
            segment_id: "s1".into(), origin_task_id: None, destination_task_id: None,
            distance_nm: 10.0, track_deg: 0.0, resource_id: "uav1".into(), dependencies: vec![],
        }];
        let schedule: Vec<TaskScheduleEntry> = vec![];
        let wind = WindVector { speed_kts: 10.0, direction_deg: 0.0 };
        let res = predict_mission_duration(&tasks, &transits, &schedule, 100.0, &wind, dummy_obs());
        assert!(matches!(res.predicted_value, StateValue::Float { value, .. } if value > 0.0));
    }

    #[test]
    fn test_adversarial_crosswind() {
        let tasks: Vec<Task> = vec![];
        let transits = vec![TransitSegment {
            segment_id: "s1".into(), origin_task_id: None, destination_task_id: None,
            distance_nm: 10.0, track_deg: 0.0, resource_id: "uav1".into(), dependencies: vec![],
        }];
        let schedule: Vec<TaskScheduleEntry> = vec![];
        let wind = WindVector { speed_kts: 100.0, direction_deg: 90.0 }; 
        let res = predict_mission_duration(&tasks, &transits, &schedule, 50.0, &wind, dummy_obs());
        assert!(matches!(res.predicted_value, StateValue::Null));
    }

    #[test]
    fn test_adversarial_headwind() {
        let tasks: Vec<Task> = vec![];
        let transits = vec![TransitSegment {
            segment_id: "s1".into(), origin_task_id: None, destination_task_id: None,
            distance_nm: 10.0, track_deg: 0.0, resource_id: "uav1".into(), dependencies: vec![],
        }];
        let schedule: Vec<TaskScheduleEntry> = vec![];
        let wind = WindVector { speed_kts: 150.0, direction_deg: 0.0 }; 
        let res = predict_mission_duration(&tasks, &transits, &schedule, 100.0, &wind, dummy_obs());
        assert!(matches!(res.predicted_value, StateValue::Null));
    }

    #[test]
    fn test_concurrent_resources() {
        let tasks: Vec<Task> = vec![];
        let transits = vec![
            TransitSegment { segment_id: "s1".into(), origin_task_id: None, destination_task_id: None, distance_nm: 10.0, track_deg: 0.0, resource_id: "uav1".into(), dependencies: vec![] },
            TransitSegment { segment_id: "s2".into(), origin_task_id: None, destination_task_id: None, distance_nm: 10.0, track_deg: 0.0, resource_id: "uav2".into(), dependencies: vec![] },
        ];
        let schedule: Vec<TaskScheduleEntry> = vec![];
        let wind = WindVector { speed_kts: 0.0, direction_deg: 0.0 };
        let res = predict_mission_duration(&tasks, &transits, &schedule, 100.0, &wind, dummy_obs());
        assert!(matches!(res.predicted_value, StateValue::Float { value, .. } if (value - 6.0).abs() < 0.1));
    }

       #[test]
    fn test_dependency_cycle() {
        let tasks: Vec<Task> = vec![];
        let transits = vec![
            // Note: dependencies must match the generated node ID format: "transit:<segment_id>"
            TransitSegment { segment_id: "s1".into(), origin_task_id: None, destination_task_id: None, distance_nm: 10.0, track_deg: 0.0, resource_id: "uav1".into(), dependencies: vec!["transit:s2".into()] },
            TransitSegment { segment_id: "s2".into(), origin_task_id: None, destination_task_id: None, distance_nm: 10.0, track_deg: 0.0, resource_id: "uav1".into(), dependencies: vec!["transit:s1".into()] },
        ];
        let schedule: Vec<TaskScheduleEntry> = vec![];
        let wind = WindVector { speed_kts: 0.0, direction_deg: 0.0 };
        let res = predict_mission_duration(&tasks, &transits, &schedule, 100.0, &wind, dummy_obs());
        assert!(matches!(res.predicted_value, StateValue::Null));
    }

    #[test]
    fn test_sequential_resources_with_dependency() {
        let tasks: Vec<Task> = vec![];
        let transits = vec![
            TransitSegment { segment_id: "s1".into(), origin_task_id: None, destination_task_id: None, distance_nm: 10.0, track_deg: 0.0, resource_id: "uav1".into(), dependencies: vec![] },
            // s2 depends on s1, forcing sequential execution across different resources
            TransitSegment { segment_id: "s2".into(), origin_task_id: None, destination_task_id: None, distance_nm: 10.0, track_deg: 0.0, resource_id: "uav2".into(), dependencies: vec!["transit:s1".into()] },
        ];
        let schedule: Vec<TaskScheduleEntry> = vec![];
        let wind = WindVector { speed_kts: 0.0, direction_deg: 0.0 };
        let res = predict_mission_duration(&tasks, &transits, &schedule, 100.0, &wind, dummy_obs());
        // s1 takes 6 min. s2 depends on s1, so s2 starts at 6 min and takes 6 min, ending at 12 min.
        assert!(matches!(res.predicted_value, StateValue::Float { value, .. } if (value - 12.0).abs() < 0.1));
    }

    #[test]
    fn test_missing_resource_assignment() {
        let tasks: Vec<Task> = vec![];
        let transits = vec![TransitSegment {
            segment_id: "s1".into(), origin_task_id: None, destination_task_id: None,
            distance_nm: 10.0, track_deg: 0.0, resource_id: "".into(), dependencies: vec![],
        }];
        let schedule: Vec<TaskScheduleEntry> = vec![];
        let wind = WindVector { speed_kts: 0.0, direction_deg: 0.0 };
        let res = predict_mission_duration(&tasks, &transits, &schedule, 100.0, &wind, dummy_obs());
        assert!(matches!(res.predicted_value, StateValue::Null));
    }

    #[test]
    fn test_empty_task_and_transit_collections() {
        let tasks: Vec<Task> = vec![];
        let transits: Vec<TransitSegment> = vec![];
        let schedule: Vec<TaskScheduleEntry> = vec![];
        let wind = WindVector { speed_kts: 0.0, direction_deg: 0.0 };
        let res = predict_mission_duration(&tasks, &transits, &schedule, 100.0, &wind, dummy_obs());
        assert!(matches!(res.predicted_value, StateValue::Float { value, .. } if value == 0.0));
    }
}