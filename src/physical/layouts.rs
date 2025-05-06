// src/physical/layouts.rs

use super::interference::*;
use super::topology::*;

pub fn create_quadcopter_layout() -> PhysicalTopology {
    let mut topology = PhysicalTopology::new();

    // Add core components
    topology.add_component(Component {
        id: ComponentId::FlightController,
        position: Position {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }, // Center
        weight_g: 30.0,
        power_consumption_mw: 500.0,
        heat_generation_c: 10.0,
    });

    topology.add_component(Component {
        id: ComponentId::MainProcessor,
        position: Position {
            x: 2.0,
            y: 0.0,
            z: 0.0,
        }, // 2cm forward
        weight_g: 50.0,
        power_consumption_mw: 2000.0,
        heat_generation_c: 20.0,
    });

    topology.add_component(Component {
        id: ComponentId::PowerDistribution,
        position: Position {
            x: -2.0,
            y: 0.0,
            z: -1.0,
        }, // 2cm backward, 1cm down
        weight_g: 40.0,
        power_consumption_mw: 100.0,
        heat_generation_c: 15.0,
    });

    topology.add_component(Component {
        id: ComponentId::Battery,
        position: Position {
            x: -5.0,
            y: 0.0,
            z: -2.0,
        }, // 5cm backward, 2cm down
        weight_g: 200.0,
        power_consumption_mw: 0.0,
        heat_generation_c: 5.0,
    });

    // Add sensors
    topology.add_component(Component {
        id: ComponentId::Gps,
        position: Position {
            x: 0.0,
            y: 0.0,
            z: 3.0,
        }, // 3cm up
        weight_g: 15.0,
        power_consumption_mw: 80.0,
        heat_generation_c: 2.0,
    });

    // src/physical/layouts.rs (continued)

    topology.add_component(Component {
        id: ComponentId::Imu,
        position: Position {
            x: 0.5,
            y: 0.0,
            z: 0.0,
        }, // 0.5cm forward
        weight_g: 10.0,
        power_consumption_mw: 50.0,
        heat_generation_c: 1.0,
    });

    topology.add_component(Component {
        id: ComponentId::Camera,
        position: Position {
            x: 5.0,
            y: 0.0,
            z: -2.0,
        }, // 5cm forward, 2cm down
        weight_g: 35.0,
        power_consumption_mw: 350.0,
        heat_generation_c: 5.0,
    });

    topology.add_component(Component {
        id: ComponentId::Lidar,
        position: Position {
            x: 4.0,
            y: 0.0,
            z: -1.0,
        }, // 4cm forward, 1cm down
        weight_g: 60.0,
        power_consumption_mw: 800.0,
        heat_generation_c: 8.0,
    });

    // Add communication and radio
    topology.add_component(Component {
        id: ComponentId::RadioLink,
        position: Position {
            x: 0.0,
            y: 0.0,
            z: 2.0,
        }, // 2cm up
        weight_g: 20.0,
        power_consumption_mw: 1000.0,
        heat_generation_c: 12.0,
    });

    topology.add_component(Component {
        id: ComponentId::CommunicationHub,
        position: Position {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        }, // 1cm forward
        weight_g: 15.0,
        power_consumption_mw: 300.0,
        heat_generation_c: 4.0,
    });

    // Add motor controllers
    for i in 0..4 {
        let (x, y) = match i {
            0 => (7.0, 7.0),   // Front right
            1 => (7.0, -7.0),  // Front left
            2 => (-7.0, -7.0), // Back left
            3 => (-7.0, 7.0),  // Back right
            _ => panic!("Invalid motor index"),
        };

        topology.add_component(Component {
            id: ComponentId::MotorController(i),
            position: Position { x, y, z: 0.0 },
            weight_g: 25.0,
            power_consumption_mw: 250.0,
            heat_generation_c: 15.0,
        });
    }

    // Add sensor hub
    topology.add_component(Component {
        id: ComponentId::SensorHub,
        position: Position {
            x: 3.0,
            y: 0.0,
            z: 0.0,
        }, // 3cm forward
        weight_g: 20.0,
        power_consumption_mw: 150.0,
        heat_generation_c: 3.0,
    });

    // Create connections

    // Flight controller connections
    topology
        .connect(
            &ComponentId::FlightController,
            &ComponentId::MainProcessor,
            ConnectionType::PcbTrace {
                width_mils: 20,
                layers: 2,
            },
        )
        .unwrap();

    topology
        .connect(
            &ComponentId::FlightController,
            &ComponentId::Imu,
            ConnectionType::PcbTrace {
                width_mils: 10,
                layers: 1,
            },
        )
        .unwrap();

    topology
        .connect(
            &ComponentId::FlightController,
            &ComponentId::PowerDistribution,
            ConnectionType::Copper {
                gauge: 22,
                wires: 4,
                shielded: true,
            },
        )
        .unwrap();

    // Power distribution connections
    topology
        .connect(
            &ComponentId::PowerDistribution,
            &ComponentId::Battery,
            ConnectionType::Copper {
                gauge: 18,
                wires: 2,
                shielded: true,
            },
        )
        .unwrap();

    // Motor connections
    for i in 0..4 {
        topology
            .connect(
                &ComponentId::FlightController,
                &ComponentId::MotorController(i),
                ConnectionType::Copper {
                    gauge: 24,
                    wires: 3,
                    shielded: false,
                },
            )
            .unwrap();

        topology
            .connect(
                &ComponentId::PowerDistribution,
                &ComponentId::MotorController(i),
                ConnectionType::Copper {
                    gauge: 20,
                    wires: 2,
                    shielded: false,
                },
            )
            .unwrap();
    }

    // Main processor connections
    topology
        .connect(
            &ComponentId::MainProcessor,
            &ComponentId::SensorHub,
            ConnectionType::PcbTrace {
                width_mils: 25,
                layers: 2,
            },
        )
        .unwrap();

    topology
        .connect(
            &ComponentId::MainProcessor,
            &ComponentId::CommunicationHub,
            ConnectionType::PcbTrace {
                width_mils: 30,
                layers: 2,
            },
        )
        .unwrap();

    // Communication connections
    topology
        .connect(
            &ComponentId::CommunicationHub,
            &ComponentId::RadioLink,
            ConnectionType::Copper {
                gauge: 24,
                wires: 6,
                shielded: true,
            },
        )
        .unwrap();

    // Sensor connections
    topology
        .connect(
            &ComponentId::SensorHub,
            &ComponentId::Camera,
            ConnectionType::Copper {
                gauge: 26,
                wires: 8,
                shielded: false,
            },
        )
        .unwrap();

    topology
        .connect(
            &ComponentId::SensorHub,
            &ComponentId::Lidar,
            ConnectionType::Copper {
                gauge: 24,
                wires: 6,
                shielded: true,
            },
        )
        .unwrap();

    topology
        .connect(
            &ComponentId::SensorHub,
            &ComponentId::Gps,
            ConnectionType::Copper {
                gauge: 26,
                wires: 4,
                shielded: true,
            },
        )
        .unwrap();

    // Wireless connections
    topology
        .connect(
            &ComponentId::Gps,
            &ComponentId::RadioLink,
            ConnectionType::Wireless {
                frequency_mhz: 1575,
                power_mw: 100.0,
            },
        )
        .unwrap();

    topology
}

pub fn create_emc_profile() -> EmcProperties {
    let mut emc = EmcProperties::new();

    // Add emission sources

    // Motors generate strong magnetic fields
    for i in 0..4 {
        emc.add_emission_source(EmissionSource {
            component: ComponentId::MotorController(i),
            emission_type: EmissionType::Magnetic,
            strength: 8.0,
            falloff_rate: 2.0, // Inverse square law
        });
    }

    // Radio link generates RF
    emc.add_emission_source(EmissionSource {
        component: ComponentId::RadioLink,
        emission_type: EmissionType::RadioFrequency(2400),
        strength: 7.0,
        falloff_rate: 2.0,
    });

    // Power distribution generates electrical noise
    emc.add_emission_source(EmissionSource {
        component: ComponentId::PowerDistribution,
        emission_type: EmissionType::Electrical,
        strength: 6.0,
        falloff_rate: 1.8,
    });

    // Set susceptibility

    // GPS is highly susceptible to RF interference
    emc.set_susceptibility(ComponentId::Gps, EmissionType::RadioFrequency(2400), 0.9);

    // IMU is susceptible to magnetic fields
    emc.set_susceptibility(ComponentId::Imu, EmissionType::Magnetic, 0.8);

    // Most digital systems are somewhat susceptible to electrical noise
    for id in [
        ComponentId::FlightController,
        ComponentId::MainProcessor,
        ComponentId::SensorHub,
        ComponentId::Camera,
        ComponentId::Lidar,
    ]
    .iter()
    {
        emc.set_susceptibility(id.clone(), EmissionType::Electrical, 0.5);
    }

    emc
}
