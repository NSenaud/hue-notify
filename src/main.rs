use huelib::resource::{light, Alert, Modifier, ModifierType};
use huelib::color::Color;
use huelib::Bridge;
use std::net::{IpAddr, Ipv4Addr};
use std::thread::sleep;
use std::time::Duration;
use std::env;

struct PagerDuty<'a> {
    token: &'a str,
    team_id: &'a str,
}

struct Hue<'a> {
    token: &'a str,
    light_id: &'a str,
}

fn main() {
    let pagerduty_token = env::var("PAGERDUTY_TOKEN").unwrap();
    let pagerduty_team_id = env::var("PAGERDUTY_TEAM_ID").unwrap();
    let huebridge_token = env::var("HUEBRIDGE_TOKEN").unwrap();
    let huebridge_light = env::var("HUEBRIDGE_LIGHT").unwrap();

    let pagerduty = PagerDuty::new(&pagerduty_token, &pagerduty_team_id);
    let hue = Hue::new(&huebridge_token, &huebridge_light);

    loop {
        if pagerduty.get_incidents_count() > 0 {
            hue.notify();
        } else {
            println!("No opened incident");
        }
        sleep(Duration::new(60, 0));
    }
}

impl<'a> PagerDuty<'a> {
    fn new(token: &'a str, team_id: &'a str) -> PagerDuty<'a> {
        PagerDuty {
            token: token,
            team_id: team_id,
        }
    }

    fn get_incidents_count(&self) -> usize {
        let resp = ureq::get("https://api.pagerduty.com/incidents")
            .query("statuses[]", "triggered")
            .query("team_ids[]", self.team_id)
            .set("Authorization", &format!("Token token={}", self.token))
            .call();

        let resp_value = resp.into_json().unwrap();
        let incidents = resp_value["incidents"].as_array().unwrap();

        incidents.len()
    }
}

impl<'a> Hue<'a> {
    fn new(token: &'a str, light_id: &'a str) -> Hue<'a> {
        Hue {
            token: token,
            light_id: light_id,
        }
    }

    fn notify(&self) {
        // Create a bridge with IP address and username.
        let bridge = Bridge::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 24)),
            self.token
        );

        // get current setup to reapply later
        let state = bridge.get_light(self.light_id).unwrap().state;

        // Set to PagerDuty color
        let color = light::StateModifier::new()
            .on(true)
            .brightness(ModifierType::Override, 254)
            .color(Color::from_rgb(21, 163, 69));

        // Blink the light
        let alert = light::StateModifier::new()
            .alert(Alert::LSelect);

        match bridge.set_light_state(self.light_id, &color) {
            Ok(v) => v.iter().for_each(|response| println!("{}", response)),
            Err(e) => eprintln!("Failed to modify the light state: {}", e),
        };

        match bridge.set_light_state(self.light_id, &alert) {
            Ok(v) => v.iter().for_each(|response| println!("{}", response)),
            Err(e) => eprintln!("Failed to modify the light state: {}", e),
        };

        // TODO: use callback to support ACK and stop alert animation
        sleep(Duration::new(15, 0));

        // set back previous setup after blink animation
        match bridge.set_light_state(self.light_id, &self.modifier_from(state)) {
            Ok(v) => v.iter().for_each(|response| println!("{}", response)),
            Err(e) => eprintln!("Failed to modify the light state: {}", e),
        };
    }

    fn modifier_from(&self, state: light::State) -> light::StateModifier {
        light::StateModifier::new()
            .on(state.on.unwrap())
            .brightness(ModifierType::Override, state.brightness.unwrap())
            .hue(ModifierType::Override, state.hue.unwrap())
            .saturation(ModifierType::Override, state.saturation.unwrap())
            .transition_time(30)
    }
}
