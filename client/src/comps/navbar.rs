//! This module provides the component for the navbar.

use gloo_utils::{body, window};
use std::fmt;
use tracing::{debug, instrument, trace};
use tracing_unwrap::ResultExt;
use wasm_bindgen::JsValue;
use web_sys::{MouseEvent, Storage};
use yew::{html, Component, Html};

/// Dark mode or light mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DarkMode {
    /// Light mode.
    Light,
    /// Dark mode.
    Dark,
}

impl Default for DarkMode {
    fn default() -> Self {
        Self::Light
    }
}

impl From<dark_light::Mode> for DarkMode {
    fn from(value: dark_light::Mode) -> Self {
        match value {
            dark_light::Mode::Dark => Self::Dark,
            dark_light::Mode::Light => Self::Light,
            dark_light::Mode::Default => Self::default(),
        }
    }
}

impl From<String> for DarkMode {
    fn from(value: String) -> Self {
        match value.to_lowercase() {
            s if s == "light" => Self::Light,
            s if s == "dark" => Self::Dark,
            _ => Self::default(),
        }
    }
}

impl fmt::Display for DarkMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            DarkMode::Light => "light",
            DarkMode::Dark => "dark",
        };
        write!(f, "{s}")
    }
}

impl DarkMode {
    /// Return the opposite mode.
    fn other(&self) -> Self {
        match self {
            Self::Light => Self::Dark,
            Self::Dark => Self::Light,
        }
    }
}

/// Return the `localStorage`.
fn local_storage() -> Result<Storage, JsValue> {
    Ok(window().local_storage()?.expect(
        "The client should only be run in web environments, so localStorage should always exist",
    ))
}

/// Get the value of the `testTrackerDarkMode` key in `localStorage` if it's available.
#[instrument]
fn storage_get_dark_mode() -> Result<Option<DarkMode>, JsValue> {
    let dark_mode = local_storage()?.get_item("testTrackerDarkMode")?;
    debug!(?dark_mode, "testTrackerDarkMode from local storage");
    Ok(dark_mode.map(|mode| mode.into()))
}

/// Set the value of the `testTrackerDarkMode` key in `localStorage`.
fn storage_set_dark_mode(dark_mode: DarkMode) -> Result<(), JsValue> {
    local_storage()?.set_item("testTrackerDarkMode", &dark_mode.to_string())
}

/// Set dark mode on the body of the HTML by adding or removing the "dark" class.
fn set_dark_mode_on_body(dark_mode: DarkMode) -> Result<(), JsValue> {
    let class_list = body().class_list();
    let (old, new) = match dark_mode {
        DarkMode::Light => ("dark", "light"),
        DarkMode::Dark => ("light", "dark"),
    };

    if class_list.contains(old) {
        class_list.remove_1(old)?;
    }

    class_list.add_1(new)?;

    Ok(())
}

/// Get the stored dark mode setting or detect it, and then set that on the body.
#[instrument]
fn init_dark_mode() -> Result<DarkMode, JsValue> {
    let dark_mode: DarkMode = match storage_get_dark_mode() {
        Ok(Some(mode)) => mode,
        _ => {
            let mode = dark_light::detect().into();
            debug!(?mode, "Detected dark mode");
            storage_set_dark_mode(mode)?;
            mode
        }
    };
    set_dark_mode_on_body(dark_mode)?;
    Ok(dark_mode)
}

/// A message to send to the navbar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavbarMsg {
    /// Do nothing.
    Nothing,

    /// Toggle between light mode and dark mode
    ToggleDarkMode,
}

/// A simple navbar to go at the top of the page and manage the dark/light mode toggle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Navbar {
    /// Are we using light mode or dark mode?
    dark_mode: DarkMode,
}

impl Component for Navbar {
    type Message = NavbarMsg;
    type Properties = ();

    fn create(_ctx: &yew::Context<Self>) -> Self {
        Self {
            dark_mode: init_dark_mode().expect_or_log("Unable to initialise dark mode"),
        }
    }

    #[instrument]
    fn view(&self, ctx: &yew::Context<Self>) -> Html {
        trace!(?self.dark_mode, "Showing navbar");
        let symbol: Html = match self.dark_mode {
            DarkMode::Light => html! {
                <svg alt="" aria-hidden="true" viewBox="0 0 24 24" width="24" height="24">
                    <path fill="currentColor" d="M12,9c1.65,0,3,1.35,3,3s-1.35,3-3,3s-3-1.35-3-3S10.35,9,12,9 M12,7c-2.76,0-5,2.24-5,5s2.24,5,5,5s5-2.24,5-5 S14.76,7,12,7L12,7z M2,13l2,0c0.55,0,1-0.45,1-1s-0.45-1-1-1l-2,0c-0.55,0-1,0.45-1,1S1.45,13,2,13z M20,13l2,0c0.55,0,1-0.45,1-1 s-0.45-1-1-1l-2,0c-0.55,0-1,0.45-1,1S19.45,13,20,13z M11,2v2c0,0.55,0.45,1,1,1s1-0.45,1-1V2c0-0.55-0.45-1-1-1S11,1.45,11,2z M11,20v2c0,0.55,0.45,1,1,1s1-0.45,1-1v-2c0-0.55-0.45-1-1-1C11.45,19,11,19.45,11,20z M5.99,4.58c-0.39-0.39-1.03-0.39-1.41,0 c-0.39,0.39-0.39,1.03,0,1.41l1.06,1.06c0.39,0.39,1.03,0.39,1.41,0s0.39-1.03,0-1.41L5.99,4.58z M18.36,16.95 c-0.39-0.39-1.03-0.39-1.41,0c-0.39,0.39-0.39,1.03,0,1.41l1.06,1.06c0.39,0.39,1.03,0.39,1.41,0c0.39-0.39,0.39-1.03,0-1.41 L18.36,16.95z M19.42,5.99c0.39-0.39,0.39-1.03,0-1.41c-0.39-0.39-1.03-0.39-1.41,0l-1.06,1.06c-0.39,0.39-0.39,1.03,0,1.41 s1.03,0.39,1.41,0L19.42,5.99z M7.05,18.36c0.39-0.39,0.39-1.03,0-1.41c-0.39-0.39-1.03-0.39-1.41,0l-1.06,1.06 c-0.39,0.39-0.39,1.03,0,1.41s1.03,0.39,1.41,0L7.05,18.36z" />
                </svg>
            },
            DarkMode::Dark => html! {
                <svg alt="" aria-hidden="true" viewBox="0 0 24 24" width="24" height="24">
                    <path fill="currentColor" d="M9.37,5.51C9.19,6.15,9.1,6.82,9.1,7.5c0,4.08,3.32,7.4,7.4,7.4c0.68,0,1.35-0.09,1.99-0.27C17.45,17.19,14.93,19,12,19 c-3.86,0-7-3.14-7-7C5,9.07,6.81,6.55,9.37,5.51z M12,3c-4.97,0-9,4.03-9,9s4.03,9,9,9s9-4.03,9-9c0-0.46-0.04-0.92-0.1-1.36 c-0.98,1.37-2.58,2.26-4.4,2.26c-2.98,0-5.4-2.42-5.4-5.4c0-1.81,0.89-3.42,2.26-4.4C12.92,3.04,12.46,3,12,3L12,3z" />
                </svg>
            },
        };

        let graduation_cap = html! {
            <svg alt="" aria-hidden="true" viewBox="0 0 650 270" width="45" height="35">
                <path fill="currentColor" d="m 619.71792,88.878283 c 17.03495,5.5136 28.06309,20.553127 28.06309,38.314657 0,17.76152 -11.02814,32.80099 -28.06309,38.30478 l -115.69153,37.40805 14.72759,117.82752 c 0,35.63313 -86.64945,64.51332 -193.53995,64.51332 -106.89051,0 -193.53996,-28.88019 -193.53996,-64.51332 L 146.40166,202.90577 92.502089,185.47709 c -2.791743,3.9809 -5.120825,8.21477 -6.62203,12.91234 8.003129,5.87684 13.537414,14.88897 13.537414,25.57358 0,12.83261 -7.631028,23.70814 -18.477118,28.89985 l 18.184691,112.75719 c 1.633114,10.06938 -3.941522,19.62585 -11.430752,19.62585 H 46.617456 c -7.489281,0 -13.054068,-9.5565 -11.430752,-19.62585 l 18.19457,-112.75719 c -10.8461,-5.1917 -18.477118,-16.06734 -18.477118,-28.89985 0,-12.78139 7.590667,-23.59796 18.365832,-28.8191 1.602592,-7.02563 4.062595,-13.72934 7.388872,-19.95861 L 30.710568,165.50771 C 13.675625,159.99411 2.6474774,144.95457 2.6474774,127.19305 c 0,-17.76153 11.0281476,-32.800987 28.0729696,-38.314657 L 297.8459,2.5104826 c 17.95335,-5.79612 36.79275,-5.79612 54.74611,0 z m -133.74417,229.698047 -13.18501,-105.57 -120.19637,38.859 c -28.81911,9.3045 -52.12475,0.84657 -54.7461,0 l -120.19537,-38.859 -13.1958,105.57 c 10.84701,11.42097 65.96527,34.41342 160.75913,34.41342 94.79385,0 149.91282,-22.99255 160.75912,-34.41342 z m 123.78493,-183.7424 c 7.96277,-2.58108 7.43907,-12.87292 -0.0108,-15.28179 L 342.62243,33.185243 c -6.77363,-2.18732 -19.61608,-4.91902 -34.83679,0 L 40.660153,119.55214 c -7.519793,2.42949 -7.912569,12.70066 0,15.27192 l 42.830996,13.84949 c 9.646092,-7.23727 20.845571,-12.61013 33.345431,-14.94803 L 322.25,95.218933 c 8.56818,-1.63312 17.17667,4.10198 18.80987,12.871917 1.64295,8.76013 -4.12265,17.18655 -12.8818,18.81964 l -197.46216,37.03488 177.07898,57.25537 c 11.40121,3.68951 23.43647,3.68951 34.83769,0 z" />
            </svg>
        };

        let onclick = ctx.link().callback(|event: MouseEvent| {
            if event.detail() == 0 {
                NavbarMsg::Nothing
            } else {
                NavbarMsg::ToggleDarkMode
            }
        });

        let text = format!(
            "Toggle to {new} mode (currently {current} mode)",
            new = self.dark_mode.other(),
            current = self.dark_mode
        );

        html! {
            <navbar>
                <h1>
                    <span id="graduation-cap" role="img" aria-hidden="true"> { graduation_cap } </span>
                    { "TestTracker" }
                </h1>
                <button id="toggle-dark-mode" aria-label={text.clone()} title={text} {onclick}>
                    { symbol }
                </button>
            </navbar>
        }
    }

    #[instrument]
    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            NavbarMsg::Nothing => false,
            NavbarMsg::ToggleDarkMode => {
                trace!(starting_mode = ?self.dark_mode, "Toggling dark mode");
                self.dark_mode = self.dark_mode.other();
                storage_set_dark_mode(self.dark_mode)
                    .expect_or_log("We should be able to set the testTrackerDarkMode value");
                set_dark_mode_on_body(self.dark_mode)
                    .expect_or_log("We should be able to change the dark mode class on the body");
                trace!(ending_mode = ?self.dark_mode, "Toggled dark mode");
                true
            }
        }
    }
}
