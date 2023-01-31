#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use windows::w;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{SendMessageTimeoutW, SMTO_BLOCK, WM_SETTINGCHANGE};
use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_WRITE};
use winreg::RegKey;

const PERSONALIZE_REGISTRY_KEY: &str =
    r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";
const APPS_USE_LIGHT_THEME_REGISTRY_KEY: &str = "AppsUseLightTheme";
const SYSTEM_USES_LIGHT_THEME_REGISTRY_KEY: &str = "SystemUsesLightTheme";

const SYSTEM_REQUIREMENTS_ERROR_MESSAGE: &str = "This program requires Windows 10 14393 or above";

#[derive(Debug)]
enum Theme {
    Light,
    Dark,
}

fn main() {
    if !cfg!(windows) {
        panic!("{SYSTEM_REQUIREMENTS_ERROR_MESSAGE}");
    }

    let personalize_reg_key = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey_with_flags(PERSONALIZE_REGISTRY_KEY, KEY_READ | KEY_WRITE)
        .expect(SYSTEM_REQUIREMENTS_ERROR_MESSAGE);

    let current_windows_theme = get_windows_theme(&personalize_reg_key);
    println!("Current Windows theme is: {current_windows_theme:?}");

    let theme_to_switch_to = match current_windows_theme {
        Theme::Light => Theme::Dark,
        Theme::Dark => Theme::Light,
    };
    println!("Setting Windows theme to: {theme_to_switch_to:?}");
    set_windows_theme(theme_to_switch_to, &personalize_reg_key);
}

fn get_windows_theme(personalize_reg_key: &RegKey) -> Theme {
    let theme_reg_value: u32 = personalize_reg_key
        .get_value(APPS_USE_LIGHT_THEME_REGISTRY_KEY)
        .expect(SYSTEM_REQUIREMENTS_ERROR_MESSAGE);

    match theme_reg_value {
        0 => Theme::Dark,
        _ => Theme::Light,
    }
}

fn set_windows_theme(theme: Theme, personalize_reg_key: &RegKey) {
    let reg_value: u32 = match theme {
        Theme::Light => 1,
        Theme::Dark => 0,
    };

    personalize_reg_key
        .set_value(APPS_USE_LIGHT_THEME_REGISTRY_KEY, &reg_value)
        .expect(SYSTEM_REQUIREMENTS_ERROR_MESSAGE);

    if personalize_reg_key
        .get_value::<u32, _>(SYSTEM_USES_LIGHT_THEME_REGISTRY_KEY)
        .is_ok()
    {
        personalize_reg_key
            .set_value(SYSTEM_USES_LIGHT_THEME_REGISTRY_KEY, &reg_value)
            .expect(SYSTEM_REQUIREMENTS_ERROR_MESSAGE);
    }

    broadcast_windows_theme_changed_message();
}

fn broadcast_windows_theme_changed_message() {
    unsafe {
        SendMessageTimeoutW(
            HWND(0xffff), // HWND_BROADCAST
            WM_SETTINGCHANGE,
            WPARAM::default(),
            LPARAM(w!("ImmersiveColorSet").as_ptr() as isize),
            SMTO_BLOCK,
            100,
            None,
        );
    }
}
