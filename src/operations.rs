impl UavSystems {
    pub fn execute_mission(&mut self) {
        // 1. Establish communications
        self.comms.add_operator("CTRL-1".into(), 3);
        
        // 2. Activate payload
        self.payload.activate();
        
        // 3. Handle radar contacts
        self.comms.update_radar(vec![
            RadarContact {
                distance_m: 1500.0,
                bearing_deg: 45.0,
                relative_speed_mps: -12.5,
            }
        ]);
        
        // 4. Navigation
        self.comms.log_beacon(NavigationBeacon {
            id: "NAV-1".into(),
            position: (34.0522, -118.2437), // LA coordinates
            signal_strength: 0.85,
        });
    }
}