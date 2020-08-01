#[macro_use]
extern crate log;

use std::env;
use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;

use anyhow::Result;
use env_logger::Env;
use futures::executor::block_on;
use huelib::color::Color;
use huelib::resource::{light, Alert, Modifier, ModifierType};
use huelib::Bridge;

const TRANSITION_TIME: u16 = 10;

/// PagerDuty notifications support
///
/// * token: a PagerDuty token with read access
/// * team_id: team ID to look for unacknowledged alerts
/// * user_id: user ID to filter alerts for
/// * color: Hue alert color
struct PagerDuty {
    token: String,
    team_id: String,
    user_id: String,
    color: Color,
}

//#[derive(Clone)]
struct Hue {
    light_id: String,
    bridge: Bridge,
}

fn main() {
    env_logger::from_env(Env::default().default_filter_or("info")).init();

    info!("Initialazing...");
    let token = env::var("PAGERDUTY_TOKEN").unwrap();
    let team_id = env::var("PAGERDUTY_TEAM_ID").unwrap();
    let user_id = env::var("PAGERDUTY_USER_ID").unwrap();
    let ip = env::var("HUEBRIDGE_IP").unwrap();
    let username = env::var("HUEBRIDGE_USERNAME").unwrap();
    let light = env::var("HUEBRIDGE_LIGHT").unwrap();

    let pagerduty = PagerDuty::new(token, team_id, user_id);
    let hue = Hue::new(Ipv4Addr::from_str(&ip).unwrap(), username, light);

    let future = run(pagerduty, hue);
    block_on(future);
}

async fn run(pagerduty: PagerDuty, hue: Hue) {
    info!("Starting up...");
    match hue.blink(pagerduty.color) {
        Ok(_) => (),
        Err(e) => error!("Notification error: {}", e),
    };

    loop {
        info!("Looking for new alerts...");
        let check = check_and_notify(&pagerduty, &hue);
        let sleep = wait_async(59);

        futures::join!(check, sleep);
    }
}

fn wait(seconds: u64) {
    debug!("Wait for {}s...", seconds);
    sleep(Duration::new(seconds, 0))
}

async fn wait_async(seconds: u64) {
    debug!("Wait for {}s... (async)", seconds);
    sleep(Duration::new(seconds, 0))
}

async fn check_and_notify(pagerduty: &PagerDuty, hue: &Hue) {
    if pagerduty.get_incidents_count() > 0 {
        info!("New PagerDuty incident triggered!");
        match hue.alert(pagerduty.color) {
            Ok(_) => (),
            Err(e) => error!("Notification error: {}", e),
        };
    } else {
        debug!("No new triggered incident");
    }
}

impl PagerDuty {
    fn new(token: String, team_id: String, user_id: String) -> PagerDuty {
        info!(
            "New PagerDuty configuration for team ID {} and user ID {}",
            team_id, user_id
        );
        PagerDuty {
            token,
            team_id,
            user_id,
            color: Color::from_rgb(21, 163, 69),
        }
    }

    fn get_incidents_count(&self) -> usize {
        debug!("Looking for triggered incidents on PagerDuty...");
        let resp = ureq::get("https://api.pagerduty.com/incidents")
            .query("statuses[]", "triggered")
            .query("team_ids[]", &self.team_id)
            .query("user_ids[]", &self.user_id)
            .set("Authorization", &format!("Token token={}", self.token))
            .call();

        let resp_value = resp.into_json().unwrap();
        let incidents = resp_value["incidents"].as_array().unwrap();

        debug!(
            "Found {} trigeered incidents on PagerDuty.",
            incidents.len()
        );
        incidents.len()
    }
}

impl Hue {
    fn new(ip: Ipv4Addr, username: String, light_id: String) -> Hue {
        info!(
            "New Hue Brigde configuration at {} for light {}",
            ip, light_id
        );
        Hue {
            bridge: Bridge::new(IpAddr::V4(ip), &username),
            light_id,
        }
    }

    fn blink(&self, color: Color) -> Result<()> {
        info!("Blinking...");
        let alert = Alert::Select;
        let duration = 1;
        self.notify(alert, color, duration)?;
        debug!("Done.");
        Ok(())
    }

    fn alert(&self, color: Color) -> Result<()> {
        info!("Alerting...");
        let alert = Alert::LSelect;
        let duration = 15;
        self.notify(alert, color, duration)?;
        debug!("Done.");
        Ok(())
    }

    fn notify(&self, alert: Alert, color: Color, duration: u64) -> Result<()> {
        // Get current setup to reapply later
        let light = self.bridge.get_light(&self.light_id)?;
        let state = light.state;

        // Set to PagerDuty color
        let color_modifier = light::StateModifier::new()
            .on(true)
            .brightness(ModifierType::Override, 254)
            .color(color)
            .transition_time(TRANSITION_TIME);

        match self.bridge.set_light_state(&self.light_id, &color_modifier) {
            Ok(v) => v.iter().for_each(|response| debug!("{}", response)),
            Err(e) => error!("Failed to modify the light state: {}", e),
        };

        wait((TRANSITION_TIME / 10) as u64);

        // Blink the light
        let alert = light::StateModifier::new().alert(alert);
        match self.bridge.set_light_state(&self.light_id, &alert) {
            Ok(v) => v.iter().for_each(|response| debug!("{}", response)),
            Err(e) => error!("Failed to modify the light state: {}", e),
        };

        wait(duration);

        // set back previous setup after blink animation
        match self
            .bridge
            .set_light_state(&self.light_id, &self.modifier_from(state))
        {
            Ok(v) => v.iter().for_each(|response| debug!("{}", response)),
            Err(e) => error!("Failed to modify the light state: {}", e),
        };

        Ok(())
    }

    fn modifier_from(&self, state: light::State) -> light::StateModifier {
        light::StateModifier::new()
            .on(state.on.unwrap())
            .brightness(ModifierType::Override, state.brightness.unwrap())
            .hue(ModifierType::Override, state.hue.unwrap())
            .saturation(ModifierType::Override, state.saturation.unwrap())
            .transition_time(TRANSITION_TIME)
    }
}
