#[test]
fn test_full_ooda_cycle() {
    let mut comms = CommunicationHub::new(
        LinkType::MAVLink {
            version: 2,
            heartbeat_interval_ms: 1000,
        },
        true
    ).clone();
    let mut payload = PayloadManager::new(/* ... */);
    let mut flight = FlightController::new();
    let mut ooda = OodaLoop::new();
    
    // Simulate radar contact
    comms.update_radar(vec![RadarContact {
        distance_m: 1000.0,
        bearing_deg: 45.0,
        relative_speed_mps: -25.0,
        via_link: LinkType::MAVLink { /* ... */ },
    }]);
    
    let cycle_time = ooda.execute_cycle(&mut comms, &mut payload, &mut flight);
    
    assert!(cycle_time < Duration::from_millis(500));
    assert_eq!(comms.process_ooda_cycle(&ooda), CommsPriority::High);
    assert!(payload.is_operational());
}