#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use std::{
    error::Error,
    fs,
    io::Cursor,
    path::PathBuf,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use auto_launch::{AutoLaunch, AutoLaunchBuilder};
use directories::ProjectDirs;
use discord_rich_presence::{DiscordIpc, DiscordIpcClient, activity};
use serde::{Deserialize, Serialize};
use tray_icon::{
    Icon, TrayIcon, TrayIconBuilder,
    menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem},
};
use winit::{
    application::ApplicationHandler,
    event::StartCause,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
};

#[cfg(target_os = "macos")]
use winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS};

const APP_NAME: &str = "Discord Mafia";
const RETRY_INTERVAL: Duration = Duration::from_secs(10);
const DEFAULT_CLIENT_ID: &str = "1143833637767348304";
const DEFAULT_DETAILS: &str = "Wanna play all things mafia?";
const INVITE_URL: &str = "https://discord.gg/social-deduction";
const TRAY_ICON: &[u8] = include_bytes!("../assets/icon.png");

type AppResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
struct Config {
    client_id: Option<String>,
    details: String,
    state: String,
    enabled: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            client_id: None,
            details: DEFAULT_DETAILS.into(),
            state: String::new(),
            enabled: true,
        }
    }
}

struct Presence {
    client: Option<DiscordIpcClient>,
    started_at: i64,
}

impl Presence {
    fn enable(&mut self, config: &Config) -> AppResult<()> {
        self.disable();

        let client_id = config.client_id.as_deref().unwrap_or(DEFAULT_CLIENT_ID);
        let mut client = DiscordIpcClient::new(client_id);
        client.connect()?;
        let mut rich_presence = activity::Activity::new()
            .details(&config.details)
            .timestamps(activity::Timestamps::new().start(self.started_at))
            .buttons(vec![activity::Button::new(
                "Join Discord Mafia",
                INVITE_URL,
            )])
            .assets(
                activity::Assets::new()
                    .large_image("discordmafia")
                    .large_text("Join Discord Mafia")
                    .small_image("cog_icon")
                    .small_text("We have our own bot!"),
            );
        if !config.state.is_empty() {
            rich_presence = rich_presence.state(&config.state);
        }
        client.set_activity(rich_presence)?;
        self.client = Some(client);
        Ok(())
    }

    fn disable(&mut self) {
        if let Some(mut client) = self.client.take() {
            let _ = client.clear_activity();
            let _ = client.close();
        }
    }

    fn is_connected(&self) -> bool {
        self.client.is_some()
    }
}

struct TrayControls {
    _icon: TrayIcon,
    status: MenuItem,
    toggle_presence: MenuItem,
    launch_at_login: CheckMenuItem,
    reconnect: MenuItem,
    quit: MenuItem,
}

struct App {
    config: Config,
    config_path: PathBuf,
    auto_launch: AutoLaunch,
    presence: Presence,
    tray: Option<TrayControls>,
    last_retry: Instant,
}

impl App {
    fn new(config: Config, config_path: PathBuf) -> AppResult<Self> {
        Ok(Self {
            config,
            config_path,
            auto_launch: auto_launch()?,
            presence: Presence {
                client: None,
                started_at: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64,
            },
            tray: None,
            last_retry: Instant::now() - RETRY_INTERVAL,
        })
    }

    fn initialize_tray(&mut self) -> AppResult<()> {
        let menu = Menu::new();
        let status = MenuItem::new("Discord: starting…", false, None);
        let toggle_presence = MenuItem::new(
            if self.config.enabled {
                "Disable Rich Presence"
            } else {
                "Enable Rich Presence"
            },
            true,
            None,
        );
        let launch_at_login = CheckMenuItem::new(
            "Launch at login",
            true,
            self.auto_launch.is_enabled().unwrap_or(false),
            None,
        );
        let reconnect = MenuItem::new("Reconnect to Discord", true, None);
        let quit = MenuItem::new("Quit", true, None);

        menu.append(&status)?;
        menu.append(&PredefinedMenuItem::separator())?;
        menu.append(&toggle_presence)?;
        menu.append(&reconnect)?;
        menu.append(&PredefinedMenuItem::separator())?;
        menu.append(&launch_at_login)?;
        menu.append(&PredefinedMenuItem::separator())?;
        menu.append(&quit)?;

        let icon = TrayIconBuilder::new()
            .with_tooltip(APP_NAME)
            .with_icon(app_icon()?)
            .with_menu(Box::new(menu))
            .build()?;

        self.tray = Some(TrayControls {
            _icon: icon,
            status,
            toggle_presence,
            launch_at_login,
            reconnect,
            quit,
        });
        self.refresh_presence();
        Ok(())
    }

    fn refresh_presence(&mut self) {
        if !self.config.enabled {
            self.presence.disable();
            self.set_status("Discord Mafia: presence disabled");
            return;
        }

        self.last_retry = Instant::now();
        match self.presence.enable(&self.config) {
            Ok(()) => self.set_status("Discord Mafia: Rich Presence active"),
            Err(error) => self.set_status(&format!("Discord Mafia: waiting ({error})")),
        }
    }

    fn retry_presence_if_needed(&mut self) {
        if self.config.enabled
            && !self.presence.is_connected()
            && self.last_retry.elapsed() >= RETRY_INTERVAL
        {
            self.refresh_presence();
        }
    }

    fn set_status(&self, text: &str) {
        if let Some(tray) = &self.tray {
            tray.status.set_text(text);
        }
    }

    fn handle_menu_event(&mut self, event: MenuEvent) -> bool {
        let Some(tray) = &self.tray else {
            return false;
        };

        if event.id == tray.toggle_presence.id() {
            self.config.enabled = !self.config.enabled;
            if let Err(error) = save_config(&self.config_path, &self.config) {
                self.config.enabled = !self.config.enabled;
                self.set_status(&format!("Could not save settings: {error}"));
                return false;
            }

            tray.toggle_presence.set_text(if self.config.enabled {
                "Disable Rich Presence"
            } else {
                "Enable Rich Presence"
            });
            self.refresh_presence();
        } else if event.id == tray.reconnect.id() {
            self.refresh_presence();
        } else if event.id == tray.launch_at_login.id() {
            let result = if tray.launch_at_login.is_checked() {
                self.auto_launch.enable()
            } else {
                self.auto_launch.disable()
            };
            if let Err(error) = result {
                tray.launch_at_login
                    .set_checked(!tray.launch_at_login.is_checked());
                self.set_status(&format!("Could not change login launch: {error}"));
            }
        } else if event.id == tray.quit.id() {
            self.presence.disable();
            return true;
        }

        false
    }
}

fn main() -> AppResult<()> {
    let config_path = config_path()?;
    let mut config = load_config(&config_path)?;
    if apply_arguments(&mut config)? {
        save_config(&config_path, &config)?;
    }

    #[cfg(target_os = "macos")]
    let event_loop = {
        let mut builder = EventLoop::builder();
        builder.with_activation_policy(ActivationPolicy::Accessory);
        builder.build()?
    };
    #[cfg(not(target_os = "macos"))]
    let event_loop = EventLoop::new()?;
    let mut app = App::new(config, config_path)?;
    event_loop.run_app(&mut app)?;

    Ok(())
}

impl ApplicationHandler for App {
    fn resumed(&mut self, _: &ActiveEventLoop) {}

    fn window_event(
        &mut self,
        _: &ActiveEventLoop,
        _: winit::window::WindowId,
        _: winit::event::WindowEvent,
    ) {
    }

    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        event_loop.set_control_flow(ControlFlow::WaitUntil(
            Instant::now() + Duration::from_secs(1),
        ));

        if matches!(cause, StartCause::Init)
            && let Err(error) = self.initialize_tray()
        {
            eprintln!("failed to initialize {APP_NAME}: {error}");
            event_loop.exit();
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        while let Ok(event) = MenuEvent::receiver().try_recv() {
            if self.handle_menu_event(event) {
                event_loop.exit();
                return;
            }
        }
        self.retry_presence_if_needed();
    }
}

fn auto_launch() -> AppResult<AutoLaunch> {
    let executable = std::env::current_exe()?;
    let executable = executable.to_string_lossy();
    Ok(AutoLaunchBuilder::new()
        .set_app_name(APP_NAME)
        .set_app_path(&executable)
        .build()?)
}

fn config_path() -> AppResult<PathBuf> {
    let directories = ProjectDirs::from("rs", "Mafia Engine", "Mafia Engine")
        .ok_or("could not determine the user configuration directory")?;
    Ok(directories.config_dir().join("discord-rich-presence.toml"))
}

fn load_config(path: &PathBuf) -> AppResult<Config> {
    match fs::read_to_string(path) {
        Ok(contents) => Ok(toml::from_str(&contents)?),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Config::default()),
        Err(error) => Err(error.into()),
    }
}

fn save_config(path: &PathBuf, config: &Config) -> AppResult<()> {
    let parent = path
        .parent()
        .ok_or("configuration path has no parent directory")?;
    fs::create_dir_all(parent)?;
    fs::write(path, toml::to_string_pretty(config)?)?;
    Ok(())
}

fn apply_arguments(config: &mut Config) -> AppResult<bool> {
    let mut changed = false;
    let mut arguments = std::env::args().skip(1);

    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--client-id" => {
                config.client_id = Some(next_argument(&mut arguments, "--client-id")?);
                changed = true;
            }
            "--details" => {
                config.details = next_argument(&mut arguments, "--details")?;
                changed = true;
            }
            "--state" => {
                config.state = next_argument(&mut arguments, "--state")?;
                changed = true;
            }
            "--disabled" => {
                config.enabled = false;
                changed = true;
            }
            "--help" | "-h" => {
                println!(
                    "{APP_NAME}\n\nUsage:\n  mafia-discord-rich-presence [--client-id ID] [--details TEXT] [--state TEXT] [--disabled]"
                );
                std::process::exit(0);
            }
            unknown => return Err(format!("unknown argument: {unknown}").into()),
        }
    }

    Ok(changed)
}

fn next_argument(arguments: &mut impl Iterator<Item = String>, flag: &str) -> AppResult<String> {
    arguments
        .next()
        .ok_or_else(|| format!("{flag} requires a value").into())
}

fn app_icon() -> AppResult<Icon> {
    let decoder = png::Decoder::new(Cursor::new(TRAY_ICON));
    let mut reader = decoder.read_info()?;
    let buffer_size = reader
        .output_buffer_size()
        .ok_or("could not determine the decoded tray icon size")?;
    let mut pixels = vec![0; buffer_size];
    let image = reader.next_frame(&mut pixels)?;

    if image.color_type != png::ColorType::Rgba {
        return Err("the tray icon must use RGBA pixels".into());
    }

    pixels.truncate(image.buffer_size());
    Ok(Icon::from_rgba(pixels, image.width, image.height)?)
}

#[cfg(test)]
mod tests {
    use super::app_icon;

    #[test]
    fn embedded_tray_icon_is_valid() {
        app_icon().expect("the embedded tray icon should decode as RGBA PNG");
    }
}
