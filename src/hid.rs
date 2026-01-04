use hidapi::{HidApi, HidDevice};
use std::thread;
use std::time::{Duration, Instant};
use winit::event_loop::EventLoopProxy;

use crate::UserEvent;

// URL:      chrome://device-log/
// USB:      vendorId=14139, productId=4397, name='ATK A9 PRO'
// Wireless: vendorId=14139, productId=4472, name='Wireless mouse dongle- 8k NANO'
pub const VID: u16 = 14139;
pub const PID_USB: u16 = 4397;
pub const PID_WIRELESS: u16 = 4472;

const CMD_GET_CONFIG_DATA: u8 = 130;
const CMD_STATUS_FA: u8 = 0xFA;

const REPORT_LEN: usize = 64;
const READ_TIMEOUT_MS: i32 = 2000;

const NORMAL_POLL: Duration = Duration::from_secs(300);
const FAST_POLL: Duration = Duration::from_secs(2);
const QUIET_AFTER: Duration = Duration::from_secs(10);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BatteryState {
    pub percent: u8,
    pub charging: bool,
}

fn parse_battery(report: &[u8]) -> Option<BatteryState> {
    if report.len() < 9 {
        return None;
    }
    let cmd = report[0];
    if cmd != CMD_GET_CONFIG_DATA && cmd != CMD_STATUS_FA {
        return None;
    }

    let raw = report[8];
    let charging = (raw & 0x80) != 0;
    let mut percent = raw & 0x7F;
    if percent > 100 {
        percent = 100;
    }

    Some(BatteryState { percent, charging })
}

fn send_get_config(dev: &HidDevice) -> hidapi::HidResult<()> {
    // hidapi: byte0 = reportId (0), byte1.. = payload
    let mut buf = [0u8; 1 + 8];
    buf[0] = 0;
    buf[1] = CMD_GET_CONFIG_DATA;
    dev.send_feature_report(&buf)?;
    Ok(())
}

fn open_working_device(api: &HidApi) -> Option<HidDevice> {
    for d in api.device_list() {
        if d.vendor_id() == VID && (d.product_id() == PID_USB || d.product_id() == PID_WIRELESS) {
            if let Ok(dev) = d.open_device(api) {
                if send_get_config(&dev).is_ok() {
                    return Some(dev);
                }
            }
        }
    }
    None
}

pub fn spawn_hid_worker(proxy: EventLoopProxy<UserEvent>) {
    thread::spawn(move || {
        loop {
            let api = match HidApi::new() {
                Ok(a) => a,
                Err(_) => {
                    thread::sleep(Duration::from_secs(2));
                    continue;
                }
            };

            let dev = match open_working_device(&api) {
                Some(d) => d,
                None => {
                    thread::sleep(Duration::from_secs(2));
                    continue;
                }
            };

            let _ = send_get_config(&dev);

            let mut last: Option<BatteryState> = None;
            let mut last_poll = Instant::now();
            let mut last_rx = Instant::now();

            'read_loop: loop {
                let mut buf = [0u8; REPORT_LEN];
                match dev.read_timeout(&mut buf, READ_TIMEOUT_MS) {
                    Ok(n) if n > 0 => {
                        if let Some(b) = parse_battery(&buf[..n]) {
                            last_rx = Instant::now();

                            if last != Some(b) {
                                last = Some(b);

                                if proxy.send_event(UserEvent::Battery(b)).is_err() {
                                    return;
                                }
                            }
                        }
                    }
                    Ok(_) => {
                    }
                    Err(_) => break 'read_loop,
                }

                let poll_every = if last.is_none() || last_rx.elapsed() > QUIET_AFTER {
                    FAST_POLL
                } else {
                    NORMAL_POLL
                };

                if last_poll.elapsed() >= poll_every {
                    if send_get_config(&dev).is_err() {
                        break 'read_loop;
                    }
                    last_poll = Instant::now();
                }
            }

            thread::sleep(Duration::from_secs(2));
        }
    });
}
