use iced::{
    button, Application, Button, Column, Command, Container, Element, HorizontalAlignment, Length,
    Row, Settings, Text,
};
use self_update::backends::github::Release;
use semver;

// TODO get it somewhere else!!
const CURRENT_PBC_VERSION: (u64, u64, u64) = (0, 10, 0);

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
}

#[derive(Debug, Clone)]
enum UpdaterMessage {
    FetchedNewerVersions(Result<Vec<String>, String>),
}

struct Updater {
    state: UpdaterState,
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
            }
        }

        Command::none()
    }

    fn view(&mut self) -> Element<Self::Message> {
        let content = match &self.state {
            UpdaterState::Fetching => Column::new().width(Length::Shrink).push(
                Text::new("Fetching versions from Github")
                    .width(Length::Shrink)
                    .size(40),
            ),
            UpdaterState::Fetched(msg) => Column::new()
                .width(Length::Shrink)
                .push(Text::new(msg).width(Length::Shrink).size(40)),

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
