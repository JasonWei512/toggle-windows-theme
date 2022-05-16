#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use windows::Win32::Foundation::HWND;
use windows::Win32::Foundation::LPARAM;
use windows::Win32::Foundation::WPARAM;
use windows::Win32::UI::WindowsAndMessaging::SMTO_BLOCK;
use windows::Win32::UI::WindowsAndMessaging::SendMessageTimeoutW;
use windows::Win32::UI::WindowsAndMessaging::WM_SETTINGCHANGE;
use windows::w;
use winreg::enums::*;
use winreg::RegKey;

const PERSONALIZE_REGISTRY_KEY: &str =
    r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";
const APPS_USE_LIGHT_THEME_REGISTRY_KEY: &str = "AppsUseLightTheme";
const SYSTEM_USES_LIGHT_THEME_REGISTRY_KEY: &str = "SystemUsesLightTheme";

const SYSTEM_REQUIREMENTS_ERROR_MESSAGE: &str = "This program requires Windows 14393 or above";

/// A simple tool for Windows (14393 or above) to switch between light and dark themes
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {}

enum Theme {
    Light,
    Dark
}

fn main() -> Result<()> {
    let _ = Args::parse();

    if !cfg!(windows) {
        return Err(anyhow!(SYSTEM_REQUIREMENTS_ERROR_MESSAGE));
    }

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let personalize_reg_key = hkcu
        .open_subkey_with_flags(PERSONALIZE_REGISTRY_KEY, KEY_READ | KEY_WRITE)
        .context(SYSTEM_REQUIREMENTS_ERROR_MESSAGE)?;

    let windows_theme = get_windows_theme(&personalize_reg_key)?;

    match windows_theme {
        Theme::Light => set_windows_theme(Theme::Dark, &personalize_reg_key)?,
        Theme::Dark => set_windows_theme(Theme::Light, &personalize_reg_key)?
    }

    Ok(())
}

fn get_windows_theme(personalize_reg_key: &RegKey) -> Result<Theme> {
    let theme_reg_value: u32 = personalize_reg_key
        .get_value(APPS_USE_LIGHT_THEME_REGISTRY_KEY)
        .context(SYSTEM_REQUIREMENTS_ERROR_MESSAGE)?;

    match theme_reg_value {
        0 => Ok(Theme::Dark),
        _ => Ok(Theme::Light)
    }
}

fn set_windows_theme(theme: Theme, personalize_reg_key: &RegKey) -> Result<()> {
    let reg_value: u32 = match theme {
        Theme::Light => 1,
        Theme::Dark => 0
    };

    personalize_reg_key
        .set_value(APPS_USE_LIGHT_THEME_REGISTRY_KEY, &reg_value)
        .context(SYSTEM_REQUIREMENTS_ERROR_MESSAGE)?;

    if personalize_reg_key.get_value::<u32, _>(SYSTEM_USES_LIGHT_THEME_REGISTRY_KEY).is_ok() {
        let _ = personalize_reg_key.set_value(SYSTEM_USES_LIGHT_THEME_REGISTRY_KEY, &reg_value);
    }

    broadcast_windows_theme_changed_message();

    Ok(())
}

fn broadcast_windows_theme_changed_message() {
    unsafe {
        let result = SendMessageTimeoutW(
            HWND(0xffff),   // HWND_BROADCAST
            WM_SETTINGCHANGE, 
            WPARAM::default(), 
            LPARAM(w!("ImmersiveColorSet").as_ptr() as isize),
            SMTO_BLOCK, 
            100, 
            None);
        
        println!("{}", result.0);
    }
}