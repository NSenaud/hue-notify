# hue-notify

Blink your Philips Hue smart light when you receive a notification.


## Status and limitations

Hue-notify requires a Hue Bridge and currently only notify unacknowledged
PagerDuty incidents.


## Compilation

Install [cargo]() through [rustup](https://rustup.rs/) and run:
```bash
cargo build --release
```

Cross-compilation is possible via [cross](https://github.com/rust-embedded/cross)
if you want to run `hue-notify` on a Raspberry Pi for instance.


## Setup

You can configure `hue-notify` from environment variables (`.env` is supported):
```
PAGERDUTY_TOKEN="changeme"
PAGERDUTY_TEAM_ID="ABCD123"
PAGERDUTY_USER_ID="1234ABC"
HUEBRIDGE_IP="192.168.1.42"
HUEBRIDGE_USERNAME="changeme"
HUEBRIDGE_LIGHT="2"
```

Please refer to Philips Hue API documentation to generate a username.


## References

* [Philips Hue API • Get started](https://developers.meethue.com/develop/get-started-2/)
