use anyhow::Context;
use tray_icon::menu::{Menu, MenuId, MenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

pub fn build_menu() -> anyhow::Result<(Menu, MenuId)> {
    let menu = Menu::new();
    let exit = MenuItem::new("Exit", true, None);
    let exit_id = exit.id().clone();

    menu.append(&exit).context("append Exit menu item")?;
    Ok((menu, exit_id))
}

pub fn build_tray(menu: Menu, icon: Icon) -> anyhow::Result<TrayIcon> {
    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("ATK Mouse Battery")
        .with_icon(icon)
        .build()
        .context("build tray icon")?;
    Ok(tray)
}

pub fn battery_icon(percent: u8, charging: bool) -> Icon {
    let (w, h) = (16usize, 16usize);
    let mut pixels = vec![0u32; w * h]; // ARGB
    render_battery_icon_argb(&mut pixels, w, h, percent, charging);

    let mut rgba = Vec::with_capacity(w * h * 4);
    for px in pixels {
        let a = ((px >> 24) & 0xFF) as u8;
        let r = ((px >> 16) & 0xFF) as u8;
        let g = ((px >> 8) & 0xFF) as u8;
        let b = (px & 0xFF) as u8;
        rgba.extend_from_slice(&[r, g, b, a]);
    }

    Icon::from_rgba(rgba, w as u32, h as u32)
        .unwrap_or_else(|_| Icon::from_rgba(vec![0u8; w * h * 4], w as u32, h as u32).unwrap())
}

fn render_battery_icon_argb(pixels: &mut [u32], w: usize, h: usize, percent: u8, charging: bool) {
    let transparent: u32 = 0x00000000;
    let outline: u32 = 0xFFFFFFFF;
    let fill_ok: u32 = if charging { 0xFFFFC107 } else { 0xFF4CAF50 };
    let fill_low: u32 = if charging { 0xFFFF9800 } else { 0xFFF44336 };

    pixels.fill(transparent);

    let mut set = |x: usize, y: usize, argb: u32| {
        if x < w && y < h {
            pixels[y * w + x] = argb;
        }
    };

    let x0 = 2usize;
    let y0 = 4usize;
    let x1 = 13usize;
    let y1 = 11usize;
    let tx0 = 14usize;
    let ty0 = 6usize;
    let ty1 = 9usize;

    for x in x0..=x1 {
        set(x, y0, outline);
        set(x, y1, outline);
    }
    for y in y0..=y1 {
        set(x0, y, outline);
        set(x1, y, outline);
    }
    for y in ty0..=ty1 {
        set(tx0, y, outline);
    }

    let inner_x0 = x0 + 1;
    let inner_y0 = y0 + 1;
    let inner_x1 = x1 - 1;
    let inner_y1 = y1 - 1;
    let inner_w = inner_x1 - inner_x0 + 1;

    let pct = (percent.min(100)) as usize;
    let filled = (inner_w * pct + 50) / 100;
    let fill_color = if pct <= 20 { fill_low } else { fill_ok };

    for y in inner_y0..=inner_y1 {
        for x in inner_x0..=inner_x1 {
            let dx = x - inner_x0;
            if dx < filled {
                set(x, y, fill_color);
            } else {
                set(x, y, 0x33FFFFFF);
            }
        }
    }

    if charging {
        let bolt: u32 = 0xFFFFFFFF;
        let pts = [(7, 5), (6, 7), (8, 7), (7, 10), (10, 7), (8, 7)];
        for (x, y) in pts {
            set(x as usize, y as usize, bolt);
        }
    }
}
