use iced::{
    button, Application, Button, Color, Column, Command, Container, Element, HorizontalAlignment,
    Length, Row, Settings, Text,
};
use self_update::backends::github::Release;
use self_update::errors::Error;
use semver;
use std::convert::TryFrom;
use std::ffi::CString;

// TODO get it somewhere else!!
const CURRENT_PBC_VERSION: (u64, u64, u64) = (0, 10, 0);

#[repr(C)]
enum UpdateCheckingStatus {
    UpdatesAvailable(u64, u64, u64), // String = latest verison
    UpToDate,
    Error,
}

extern "C" fn updates_available() -> UpdateCheckingStatus {
    if let Ok(versions) = fetch_and_parse_releases() {
        if let Some(latest_version) = versions.first() {
            let major = latest_version.major;
            let minor = latest_version.minor;
            let patch = latest_version.patch;
            UpdateCheckingStatus::UpdatesAvailable(major, minor, patch)
        } else {
            UpdateCheckingStatus::UpToDate
        }
    } else {
        UpdateCheckingStatus::Error
    }
}

enum MyError {
    SemVerError,
    SelfUpdateError,
}

impl std::convert::From<self_update::errors::Error> for MyError {
    fn from(_: self_update::errors::Error) -> Self {
        MyError::SelfUpdateError
    }
}

impl std::convert::From<semver::SemVerError> for MyError {
    fn from(_: semver::SemVerError) -> Self {
        MyError::SemVerError
    }
}

fn fetch_and_parse_releases() -> Result<Vec<semver::Version>, MyError> {
    let releases = use_self_update()?;
    let versions: Vec<semver::Version> = releases
        .iter()
        .filter_map(|release| {
            semver::Version::parse(release.version()).ok()
        })
        .filter(|version| {
            let (major, minor, patch) = CURRENT_PBC_VERSION;
            version.gt(&semver::Version::new(major, minor, patch))
        })
        .collect();

    Ok(versions)

    /*if let Ok(releases) = use_self_update() {
        println!("found releases:");
        //println!("{:#?}\n", releases);

        let release_versions: Vec<String> = releases
            .iter()
            .map(|release| semver::Version::parse(release.version()?))
            .filter(|&release| {
                let (major, minor, patch) = CURRENT_PBC_VERSION;
                semver::Version::parse(release.version())
                    .unwrap() // TODO no unwrapping here!
                    .gt(&semver::Version::new(major, minor, patch))
            })
            .map(|release| release.version().to_owned())
            .collect();
        println!("{:#?}\n", release_strings);

        if release_strings.is_empty() {
            UpdateCheckingStatus::UpToDate
        } else {
            let latest_version = semver::Version::parse(release[0])
            UpdateCheckingStatus::UpdatesAvailable(semver::Version::)

        }
    } else {
        UpdateCheckingStatus::Error
    }*/
}

pub fn main() {
    let mut settings = Settings::default();
    settings.window.size = (1024, 300);
    Updater::run(settings);
}

fn use_self_update() -> Result<Vec<Release>, self_update::errors::Error> {
    self_update::backends::github::ReleaseList::configure()
        .repo_owner("obraunsdorf")
        .repo_name("playbook-creator")
        .build()?
        .fetch()
}

async fn fetch_updates() -> Result<Vec<String>, String> {
    if let Ok(releases) = use_self_update() {
        println!("found releases:");
        //println!("{:#?}\n", releases);

        let release_strings = releases
            .iter()
            .filter(|&release| {
                let (major, minor, patch) = CURRENT_PBC_VERSION;
                semver::Version::parse(release.version())
                    .unwrap() // TODO no unwrapping here!
                    .gt(&semver::Version::new(major, minor, patch))
            })
            .map(|release| release.version().to_owned())
            .collect();
        println!("{:#?}\n", release_strings);
        Ok(release_strings)
    } else {
        Err("Some problem while fetching versions from github".to_owned())
    }
}

enum UpdaterState {
    Fetching,
    Fetched(String), // Status
    Downloading,
    Updating,
    Updated,
    Starting,
}

#[derive(Debug, Clone)]
enum GuiMessage {
    StartPBC,
}

#[derive(Debug, Clone)]
enum UpdaterMessage {
    FetchedNewerVersions(Result<Vec<String>, String>),
    GuiMessage(GuiMessage),
}

struct Updater {
    state: UpdaterState,
}

fn new_button<'a>(state: &'a mut button::State, text: &str) -> Button<'a, UpdaterMessage> {
    Button::new(state, Text::new(text).color(Color::WHITE))
        .border_radius(10)
        .padding(10)
}

impl Application for Updater {
    type Message = UpdaterMessage;

    fn new() -> (Self, Command<Self::Message>) {
        (
            Updater {
                state: UpdaterState::Fetching,
            },
            Command::perform(fetch_updates(), UpdaterMessage::FetchedNewerVersions),
        )
    }

    fn title(&self) -> String {
        String::from("PBC Updater")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            UpdaterMessage::FetchedNewerVersions(result) => {
                let fetched_msg = match result {
                    Ok(versions) => {
                        let s = "Found the following new versions: \n".to_owned();
                        versions.iter().fold(s, |agg, version| agg + version + "\n")
                    }
                    Err(err) => err,
                };
                self.state = UpdaterState::Fetched(fetched_msg);
                std::process::Command::new("PlaybookCreator")
                    .current_dir("../playbook-creator/bin")
                    .spawn();
            }

            UpdaterMessage::GuiMessage(gui_msg) => match gui_msg {
                GuiMessage::StartPBC => {
                    self.state = UpdaterState::Starting;
                }
            },
        }

        Command::none()
    }

    fn view(&mut self) -> Element<Self::Message> {
        let content = match &self.state {
            UpdaterState::Fetching => Column::new()
                .width(Length::Shrink)
                .push(
                    Text::new("Fetching versions from Github")
                        .width(Length::Shrink)
                        .size(40),
                ),
            UpdaterState::Fetched(msg) => Column::new()
                .width(Length::Shrink)
                .push(Text::new(msg).width(Length::Shrink).size(40)),

            UpdaterState::Starting => Column::new()
                .width(Length::Shrink)
                .push(Text::new("Starting PBC now...").width(Length::Shrink).size(40)),

            _ => Column::new().width(Length::Shrink).push(
                Text::new("Whoopsie, something went wrong")
                    .width(Length::Shrink)
                    .size(40),
            ),
        };

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
