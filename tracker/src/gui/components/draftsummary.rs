use async_std::path::Path;
use iced::{
    alignment::Horizontal,
    pure::{
        column, container, horizontal_space, row, text, vertical_space, widget::Container, Element,
    },
    Command, Length,
};

use crate::{
    gui::{style, TrackerMessage},
    mtgadb::{MtgaDb, UserSession},
    Result,
};

pub struct DraftSummaryComponent {
    database: MtgaDb,
    selected_set: String,
    display_user_session: Option<UserSession>,
    model: DraftSummaryModel,
}

#[derive(Default)]
pub struct DraftSummaryModel {
    packs_owned: u32,
    rares_owned: u32,
    mythics_owned: u32,
    rares_in_set: u32,
    mythics_in_set: u32,
}

impl DraftSummaryModel {
    pub fn new(database: &MtgaDb, user: &UserSession, set: &str) -> Result<DraftSummaryModel> {
        let mythics_in_set = database.get_mythic_cards_in_boosters(set)?.len() as u32 * 4;
        let rares_in_set = database.get_rare_cards_in_boosters(set)?.len() as u32 * 4;

        let rares_owned = database
            .get_collected_cards_in_boosters(user.user_id(), &set, "rare")?
            .iter()
            .fold(0_u32, |acc, e| acc + e.1);
        let mythics_owned = database
            .get_collected_cards_in_boosters(user.user_id(), &set, "mythic")?
            .iter()
            .fold(0_u32, |acc, e| acc + e.1);

        let packs_owned = 0_u32;
        Ok(DraftSummaryModel {
            packs_owned,
            rares_owned,
            mythics_owned,
            rares_in_set,
            mythics_in_set,
        })
    }

    pub fn packs_owned(&self) -> u32 {
        self.packs_owned
    }

    pub fn rares_owned(&self) -> u32 {
        self.rares_owned
    }

    pub fn rares_needed(&self) -> u32 {
        self.rares_in_set - self.rares_owned
    }

    pub fn mythics_owned(&self) -> u32 {
        self.mythics_owned
    }

    pub fn mythics_needed(&self) -> u32 {
        self.mythics_in_set - self.mythics_owned
    }

    pub fn critical_point(&self) -> f32 {
        0.0
    }

    pub fn packs_per_draft(&self) -> f32 {
        0.0
    }

    pub fn rares_per_draft(&self) -> f32 {
        0.0
    }

    pub fn mythics_per_draft(&self) -> f32 {
        0.0
    }
}

impl DraftSummaryComponent {
    pub fn new<P>(database_path: P) -> DraftSummaryComponent
    where
        P: AsRef<Path>,
    {
        DraftSummaryComponent {
            database: MtgaDb::new(database_path),
            selected_set: String::from("dmu"),
            display_user_session: None,
            model: DraftSummaryModel::default(),
        }
    }

    pub fn set_current_user(&mut self, user: UserSession) -> Result<Command<TrackerMessage>> {
        self.display_user_session = Some(user);
        self.update_state()
    }

    pub fn change_selected_set(&mut self, set: String) -> Result<Command<TrackerMessage>> {
        self.selected_set = set;
        self.update_state()
    }

    pub fn update_state(&mut self) -> Result<Command<TrackerMessage>> {
        let user = match self.display_user_session.as_ref() {
            Some(user) => user,
            None => return Err("Display user has not been set".into()),
        };

        self.model = DraftSummaryModel::new(&self.database, user, &self.selected_set)?;

        Ok(Command::none())
    }

    pub fn view(&self) -> Element<TrackerMessage> {
        let model = &self.model;
        let set_name = self.selected_set.to_uppercase();
        let rares_summary = container(
            column()
                .push(
                    container(text(format!("{} RARES", set_name)).size(24))
                        .width(Length::Fill)
                        .center_x(),
                )
                .push(
                    row()
                        .push(
                            container(text(format!("{} Packs Owned (P):", set_name)).size(20))
                                .padding(5)
                                .width(Length::FillPortion(75))
                                .align_x(Horizontal::Right),
                        )
                        .push(
                            container(text(model.packs_owned().to_string()).size(20))
                                .padding(5)
                                .width(Length::FillPortion(25))
                                .align_x(Horizontal::Center),
                        ),
                )
                .push(
                    row()
                        .push(
                            container(text(format!("{} Rares Owned (R):", set_name)).size(20))
                                .padding(5)
                                .width(Length::FillPortion(75))
                                .align_x(Horizontal::Right),
                        )
                        .push(
                            container(text(model.rares_owned().to_string()).size(20))
                                .padding(5)
                                .width(Length::FillPortion(25))
                                .align_x(Horizontal::Center),
                        ),
                )
                .push(
                    row()
                        .push(
                            container(text("Rares per Draft (N):").size(20))
                                .padding(5)
                                .width(Length::FillPortion(75))
                                .align_x(Horizontal::Right),
                        )
                        .push(
                            container(text(format!("{:.2}", model.rares_per_draft())).size(20))
                                .padding(5)
                                .width(Length::FillPortion(25))
                                .align_x(Horizontal::Center),
                        ),
                )
                .push(
                    row()
                        .push(
                            container(text("Packs per Draft (W):").size(20))
                                .padding(5)
                                .width(Length::FillPortion(75))
                                .align_x(Horizontal::Right),
                        )
                        .push(
                            container(text(format!("{:.2}", model.packs_per_draft())).size(20))
                                .padding(5)
                                .width(Length::FillPortion(25))
                                .align_x(Horizontal::Center),
                        ),
                )
                .push(
                    row()
                        .push(
                            container(text("Critical Point (drafts):").size(20))
                                .padding(5)
                                .width(Length::FillPortion(75))
                                .align_x(Horizontal::Right),
                        )
                        .push(
                            container(text(format!("{:.2}", model.critical_point())).size(20))
                                .padding(5)
                                .width(Length::FillPortion(25))
                                .align_x(Horizontal::Center),
                        ),
                )
                .push(vertical_space(Length::Units(25)))
                .push(
                    row()
                        .push(
                            container(text("Rares needed:").size(20))
                                .padding(5)
                                .width(Length::FillPortion(75))
                                .align_x(Horizontal::Right),
                        )
                        .push(
                            container(text(model.rares_needed().to_string()).size(20))
                                .padding(5)
                                .width(Length::FillPortion(25))
                                .align_x(Horizontal::Center),
                        ),
                ),
        )
        .style(style::DraftSummaryPanel)
        .width(Length::Units(275));

        let mythics_summary = container(
            column()
                .push(
                    container(text(format!("{} MYTHICS", set_name)).size(24))
                        .width(Length::Fill)
                        .center_x(),
                )
                .push(
                    row()
                        .push(
                            container(text(format!("{} Packs Owned (P):", set_name)).size(20))
                                .padding(5)
                                .width(Length::FillPortion(75))
                                .align_x(Horizontal::Right),
                        )
                        .push(
                            container(text(model.packs_owned().to_string()).size(20))
                                .padding(5)
                                .width(Length::FillPortion(25))
                                .align_x(Horizontal::Center),
                        ),
                )
                .push(
                    row()
                        .push(
                            container(text(format!("{} Mythics Owned (M):", set_name)).size(20))
                                .padding(5)
                                .width(Length::FillPortion(75))
                                .align_x(Horizontal::Right),
                        )
                        .push(
                            container(text(model.mythics_owned().to_string()).size(20))
                                .padding(5)
                                .width(Length::FillPortion(25))
                                .align_x(Horizontal::Center),
                        ),
                )
                .push(
                    row()
                        .push(
                            container(text("Mythics per Draft (N):").size(20))
                                .padding(5)
                                .width(Length::FillPortion(75))
                                .align_x(Horizontal::Right),
                        )
                        .push(
                            container(text(format!("{:.2}", model.mythics_per_draft())).size(20))
                                .padding(5)
                                .width(Length::FillPortion(25))
                                .align_x(Horizontal::Center),
                        ),
                )
                .push(
                    row()
                        .push(
                            container(text("Packs per Draft (W):").size(20))
                                .padding(5)
                                .width(Length::FillPortion(75))
                                .align_x(Horizontal::Right),
                        )
                        .push(
                            container(text(format!("{:.2}", model.packs_per_draft())).size(20))
                                .padding(5)
                                .width(Length::FillPortion(25))
                                .align_x(Horizontal::Center),
                        ),
                )
                .push(
                    row()
                        .push(
                            container(text("Critical Point (drafts):").size(20))
                                .padding(5)
                                .width(Length::FillPortion(75))
                                .align_x(Horizontal::Right),
                        )
                        .push(
                            container(text(format!("{:.2}", model.critical_point())).size(20))
                                .padding(5)
                                .width(Length::FillPortion(25))
                                .align_x(Horizontal::Center),
                        ),
                )
                .push(vertical_space(Length::Units(25)))
                .push(
                    row()
                        .push(
                            container(text("Mythics needed:").size(20))
                                .padding(5)
                                .width(Length::FillPortion(75))
                                .align_x(Horizontal::Right),
                        )
                        .push(
                            container(text(model.mythics_needed().to_string()).size(20))
                                .padding(5)
                                .width(Length::FillPortion(25))
                                .align_x(Horizontal::Center),
                        ),
                ),
        )
        .style(style::DraftSummaryPanel)
        .width(Length::Units(275));

        let general_stats: Container<TrackerMessage> = container(
            column()
                .push(
                    container(text(format!("{} STATS", set_name)).size(24))
                        .width(Length::Fill)
                        .center_x(),
                )
                .push(
                    row()
                        .push(
                            container(text("Avg Wins:").size(20))
                                .padding(5)
                                .width(Length::FillPortion(75))
                                .align_x(Horizontal::Right),
                        )
                        .push(
                            container(text("0").size(20))
                                .padding(5)
                                .width(Length::FillPortion(25))
                                .align_x(Horizontal::Center),
                        )
                        .push(horizontal_space(Length::Units(50)))
                        .push(
                            container(text("Total Spent (gems):").size(20))
                                .padding(5)
                                .width(Length::FillPortion(75))
                                .align_x(Horizontal::Right),
                        )
                        .push(
                            container(text("0").size(20))
                                .padding(5)
                                .width(Length::FillPortion(25))
                                .align_x(Horizontal::Center),
                        ),
                )
                .push(
                    row()
                        .push(
                            container(text("Win %:").size(20))
                                .padding(5)
                                .width(Length::FillPortion(75))
                                .align_x(Horizontal::Right),
                        )
                        .push(
                            container(text("0").size(20))
                                .padding(5)
                                .width(Length::FillPortion(25))
                                .align_x(Horizontal::Center),
                        )
                        .push(horizontal_space(Length::Units(50)))
                        .push(
                            container(text("Cost per Pack (gems):").size(20))
                                .padding(5)
                                .width(Length::FillPortion(75))
                                .align_x(Horizontal::Right),
                        )
                        .push(
                            container(text("0").size(20))
                                .padding(5)
                                .width(Length::FillPortion(25))
                                .align_x(Horizontal::Center),
                        ),
                )
                .push(
                    row()
                        .push(
                            container(text("Avg Rares:").size(20))
                                .padding(5)
                                .width(Length::FillPortion(75))
                                .align_x(Horizontal::Right),
                        )
                        .push(
                            container(text("0").size(20))
                                .padding(5)
                                .width(Length::FillPortion(25))
                                .align_x(Horizontal::Center),
                        )
                        .push(horizontal_space(Length::Units(50)))
                        .push(
                            container(text("Total Spent (gold):").size(20))
                                .padding(5)
                                .width(Length::FillPortion(75))
                                .align_x(Horizontal::Right),
                        )
                        .push(
                            container(text("0").size(20))
                                .padding(5)
                                .width(Length::FillPortion(25))
                                .align_x(Horizontal::Center),
                        ),
                )
                .push(
                    row()
                        .push(
                            container(text("Avg Mythics (N):").size(20))
                                .padding(5)
                                .width(Length::FillPortion(75))
                                .align_x(Horizontal::Right),
                        )
                        .push(
                            container(text("0").size(20))
                                .padding(5)
                                .width(Length::FillPortion(25))
                                .align_x(Horizontal::Center),
                        )
                        .push(horizontal_space(Length::Units(50)))
                        .push(
                            container(text("Cost per Pack (gold):").size(20))
                                .padding(5)
                                .width(Length::FillPortion(75))
                                .align_x(Horizontal::Right),
                        )
                        .push(
                            container(text("0").size(20))
                                .padding(5)
                                .width(Length::FillPortion(25))
                                .align_x(Horizontal::Center),
                        ),
                )
                .push(
                    row()
                        .push(
                            container(text("Mythic + Rares:").size(20))
                                .padding(5)
                                .width(Length::FillPortion(75))
                                .align_x(Horizontal::Right),
                        )
                        .push(
                            container(text("0").size(20))
                                .padding(5)
                                .width(Length::FillPortion(25))
                                .align_x(Horizontal::Center),
                        )
                        .push(horizontal_space(Length::Units(50)))
                        .push(
                            container(text("Drafts completed:").size(20))
                                .padding(5)
                                .width(Length::FillPortion(75))
                                .align_x(Horizontal::Right),
                        )
                        .push(
                            container(text("0").size(20))
                                .padding(5)
                                .width(Length::FillPortion(25))
                                .align_x(Horizontal::Center),
                        ),
                ),
        )
        .style(style::DraftSummaryPanel)
        .width(Length::Units(600));

        let draft_summary = container(
            column()
                .push(row().push(rares_summary).push(mythics_summary).spacing(50))
                .push(general_stats)
                .spacing(20),
        )
        .padding(20);

        let draft_panel_content = column().push(draft_summary);

        container(draft_panel_content).into()
    }
}
