use defmt_rtt as _; // global logger
use panic_probe as _;
use stm32f1xx_hal as _; // memory layout

use heapless::{String, Vec};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    pub name: String<16>,
    pub time: Vec<f32, 32>,
    pub temperature: Vec<f32, 32>,
}
#[derive(Debug, PartialEq, Serialize, Deserialize)]

pub struct Feedback {
    pub status: String<16>,
    pub time: f32,
    pub temperature: f32,
    pub error: f32,
    pub setpoint: f32,
    pub dutycycle: f32,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Start {
    pub reason: String<16>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Stop {
    pub reason: String<16>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Profile(Profile),
    Feedback(Feedback),
    Start(Start),
    Stop(Stop),
}

impl defmt::Format for Profile {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt,
            r#"
            "name": {:?},
            "time": {:?}
            "temperature": {:?}",
            "#,
            self.name.as_str(),
            self.time.as_slice(),
            self.temperature.as_slice(),
        )
    }
}

impl defmt::Format for Stop {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt,
            r#"
            "reason": "{}",
            "#,
            self.reason.as_str()
        )
    }
}

impl defmt::Format for Start {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt,
            r#"
            "reason": "{}",
            "#,
            self.reason.as_str()
        )
    }
}

impl defmt::Format for Feedback {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt,
            r#"
            "status": {:?},
            "time": {:?},
            "temperature": {:?},
            "error": {:?},
            "setpoint": {:?},
            "dutycycle": {:?},
            "#,
            self.status.as_str(),
            self.time,
            self.temperature,
            self.error,
            self.setpoint,
            self.dutycycle,
        )
    }
}

// defmt-test 0.3.0 has the limitation that this `#[tests]` attribute can only be used
// once within a crate. the module can be in any file but there can only be at most
// one `#[tests]` module in this library crate
#[cfg(test)]
#[defmt_test::tests]
mod unit_tests {
    use crate::protocol::{Profile, Protocol};
    use defmt::assert;
    use heapless::{String, Vec};

    #[test]
    fn profile_deserialize() {
        let string_message = r#"{
            "profile": {
                "name": "Some Profile",
                "time": [0.0, 0.1],
                "temperature": [30.0, 30.1]
            }
        }"#;
        let deserialize_result = serde_json_core::from_str(string_message);
        assert!(deserialize_result.is_ok());

        let profile = match deserialize_result.unwrap().0 {
            Protocol::Profile(profile) => Some(profile),
            _ => None,
        };
        assert!(profile.is_some());
        let profile = profile.unwrap();

        let expected = Profile {
            name: String::from("Some Profile"),
            time: Vec::from_slice(&[0.0, 0.1]).unwrap(),
            temperature: Vec::from_slice(&[30.0, 30.1]).unwrap(),
        };
        defmt::trace!(
            "name = {:?} == {:?}",
            expected.name.as_str(),
            profile.name.as_str()
        );
        defmt::trace!("time = {:?} == {:?}", *expected.time, *profile.time);
        defmt::trace!(
            "temperature = {:?} == {:?}",
            *expected.temperature,
            *profile.temperature
        );
        assert!(expected.name == profile.name);
        assert!(expected.time == profile.time);
        assert!(expected.temperature == profile.temperature);
    }

    #[test]
    fn profile_serialize() {
        let profile = Profile {
            name: String::from("Some Profile"),
            time: Vec::from_slice(&[0.0, 0.1]).unwrap(),
            temperature: Vec::from_slice(&[30.0, 30.1]).unwrap(),
        };

        let serialize_result =
            serde_json_core::to_string::<Protocol, 128>(&Protocol::Profile(profile));
        assert!(serialize_result.is_ok());
        let message = serialize_result.unwrap();

        let expected =
            r#"{"profile":{"name":"Some Profile","time":[0.0,0.1],"temperature":[30.0,30.1]}}"#;
        defmt::trace!("message = {:?}", message.as_str());
        defmt::trace!("expected = {:?}", expected);
        assert!(expected == message);
    }
}
