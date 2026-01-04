#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod hid;
mod tray;

use hid::BatteryState;
use tray_icon::menu::{MenuEvent, MenuId};
use tray_icon::TrayIconEvent;
use winit::application::ApplicationHandler;
use winit::event::{StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};
use winit::window::WindowId;

#[derive(Debug)]
pub enum UserEvent {
    Battery(BatteryState),
    Menu(MenuEvent),
    Wake,
}

struct App {
    tray: Option<tray_icon::TrayIcon>,
    exit_id: Option<MenuId>,
    last: Option<BatteryState>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            tray: None,
            exit_id: None,
            last: None,
        }
    }
}

impl App {
    fn init_tray(&mut self) {
        let (menu, exit_id) = match tray::build_menu() {
            Ok(v) => v,
            Err(_) => return,
        };

        let icon = tray::battery_icon(0, false);
        let tray = match tray::build_tray(menu, icon) {
            Ok(t) => t,
            Err(_) => return,
        };

        tray.set_show_menu_on_left_click(true);

        self.exit_id = Some(exit_id);
        self.tray = Some(tray);

        let _ = self
            .tray
            .as_ref()
            .unwrap()
            .set_tooltip(Some("ATK Mouse Battery: waiting for device..."));
    }

    fn update_battery(&mut self, b: BatteryState) {
        if self.last == Some(b) {
            return;
        }
        self.last = Some(b);

        if let Some(tray) = &self.tray {
            let icon = tray::battery_icon(b.percent, b.charging);
            let _ = tray.set_icon(Some(icon));

            let tip = if b.charging {
                format!("ATK Mouse Battery: {}% (charging)", b.percent)
            } else {
                format!("ATK Mouse Battery: {}%", b.percent)
            };
            let _ = tray.set_tooltip(Some(tip));
        }
    }

    fn on_menu_event(&mut self, event_loop: &ActiveEventLoop, e: MenuEvent) {
        if let Some(exit_id) = &self.exit_id {
            if &e.id == exit_id {
                event_loop.exit();
            }
        }
    }
}

impl ApplicationHandler<UserEvent> for App {
    
fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
    if self.tray.is_none() {
        self.init_tray();
    }
}

fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        if matches!(cause, StartCause::Init) {
            if self.tray.is_none() {
                self.init_tray();
            }
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::Battery(b) => self.update_battery(b),
            UserEvent::Menu(e) => self.on_menu_event(event_loop, e),
            UserEvent::Wake => {
            }
        }
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        _event: WindowEvent,
    ) {
    }
}

fn install_tray_event_forwarders(proxy: EventLoopProxy<UserEvent>) {
    let proxy_for_tray = proxy.clone();
    TrayIconEvent::set_event_handler(Some(move |e| {
        let _ = e;
        let _ = proxy_for_tray.send_event(UserEvent::Wake);
    }));

    let proxy_for_menu = proxy.clone();
    MenuEvent::set_event_handler(Some(move |e| {
        let _ = proxy_for_menu.send_event(UserEvent::Menu(e));
    }));
}

fn main() {
    let event_loop = EventLoop::<UserEvent>::with_user_event().build().unwrap();
    let proxy = event_loop.create_proxy();

    install_tray_event_forwarders(proxy.clone());

    hid::spawn_hid_worker(proxy);

    let mut app = App::default();
    let _ = event_loop.run_app(&mut app);
}
