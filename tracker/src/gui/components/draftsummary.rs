use async_std::path::Path;
use iced::{
    alignment::Horizontal,
    widget::column,
    widget::{container, horizontal_space, row, text, vertical_space},
    Command, Length,
};

use crate::{
    gui::{Element, TrackerMessage, style::container::ContainerStyle},
    mtgadb::{DraftDetails, MtgaDb, UserSession},
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
    rare_wc_owned: u32,
    mythic_wc_owned: u32,
    packs_owned: u32,
    rares_owned: u32,
    mythics_owned: u32,
    rares_in_set: u32,
    mythics_in_set: u32,
    drafts_played: Vec<DraftDetails>,
    drop_rates: (f64, f64),
}

impl DraftSummaryModel {
    pub fn new(database: &MtgaDb, user: &UserSession, set: &str) -> Result<DraftSummaryModel> {
        let rare_wc_owned = database.get_rare_wildcards(user.user_id())?;
        let mythic_wc_owned = database.get_mythic_wildcards(user.user_id())?;
        let packs_owned = database.get_packs_owned(user.user_id(), set)?;

        let drafts_played = database.get_played_drafts(user.user_id(), set)?;

        let mythics_in_set = database.get_mythic_cards_in_boosters(set)?.len() as u32 * 4;
        let rares_in_set = database.get_rare_cards_in_boosters(set)?.len() as u32 * 4;
        let rares_owned = database
            .get_collected_cards_in_boosters(user.user_id(), set, "rare")?
            .iter()
            .fold(0_u32, |sum, card_with_count| sum + card_with_count.1);
        let mythics_owned = database
            .get_collected_cards_in_boosters(user.user_id(), set, "mythic")?
            .iter()
            .fold(0_u32, |sum, card_with_count| sum + card_with_count.1);

        let drop_rates = database.get_drop_rates(set)?;

        Ok(DraftSummaryModel {
            rare_wc_owned,
            mythic_wc_owned,
            packs_owned,
            rares_owned,
            mythics_owned,
            rares_in_set,
            mythics_in_set,
            drafts_played,
            drop_rates,
        })
    }

    pub fn packs_owned(&self) -> u32 {
        self.packs_owned
    }

    pub fn rare_wc_owned(&self) -> u32 {
        self.rare_wc_owned
    }

    pub fn mythic_wc_owned(&self) -> u32 {
        self.mythic_wc_owned
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

    pub fn packs_per_draft(&self) -> f64 {
        if self.drafts_played.len() < 10 {
            1.26 // assuming 50% winrate in quick draft
        } else {
            let total_rewards = self
                .drafts_played
                .iter()
                .fold(0.0, |sum, draft_result| sum + draft_result.reward_packs());
            total_rewards / self.drafts_played.len() as f64
        }
    }

    pub fn rares_per_draft(&self) -> f64 {
        if self.drafts_played.len() < 10 {
            self.drop_rates.0 * 3.0
        } else {
            let total_rares = self.drafts_played.iter().fold(0.0, |sum, draft_result| {
                sum + draft_result.rares_collected() as f64
            });
            total_rares / self.drafts_played.len() as f64
        }
    }

    pub fn rares_critical_point(&self) -> f64 {
        const RARE_OR_MYTHIC_WILDCARD_BIAS: f64 = 28.0 / 30.0;

        (self.rares_in_set as f64
            - (self.packs_owned as f64 * self.drop_rates.0 * RARE_OR_MYTHIC_WILDCARD_BIAS)
            - self.rares_owned as f64
            - self.rare_wc_owned as f64)
            / (self.rares_per_draft()
                + (self.packs_per_draft() * self.drop_rates.0 * RARE_OR_MYTHIC_WILDCARD_BIAS))
    }

    pub fn mythics_per_draft(&self) -> f64 {
        if self.drafts_played.len() < 10 {
            self.drop_rates.1 * 3.0
        } else {
            let total_mythics = self.drafts_played.iter().fold(0.0, |sum, draft_result| {
                sum + draft_result.mythics_collected() as f64
            });
            total_mythics / self.drafts_played.len() as f64
        }
    }

    pub fn mythics_critical_point(&self) -> f64 {
        const RARE_OR_MYTHIC_WILDCARD_BIAS: f64 = 28.0 / 30.0;

        (self.mythics_in_set as f64
            - (self.packs_owned as f64 * self.drop_rates.1 * RARE_OR_MYTHIC_WILDCARD_BIAS)
            - self.mythics_owned as f64
            - self.mythic_wc_owned as f64)
            / (self.mythics_per_draft()
                + (self.packs_per_draft() * self.drop_rates.1 * RARE_OR_MYTHIC_WILDCARD_BIAS))
    }

    pub fn avg_wins(&self) -> (f64, f64) {
        let (bo1_draft_wins, bo3_draft_wins) = self.drafts_played.iter().fold(
            (0.0, 0.0),
            |(bo1_draft_wins, bo3_draft_wins), draft_result| {
                if draft_result.is_bo3() {
                    (bo1_draft_wins, bo3_draft_wins + draft_result.wins() as f64)
                } else {
                    (bo1_draft_wins + draft_result.wins() as f64, bo3_draft_wins)
                }
            },
        );
        (bo1_draft_wins, bo3_draft_wins)
    }

    pub fn win_percentage(&self) -> f64 {
        let (wins, total_games) =
            self.drafts_played
                .iter()
                .fold((0, 0), |(wins, total_games), draft| {
                    (
                        wins + draft.wins(),
                        total_games + draft.wins() + draft.losses(),
                    )
                });

        if total_games == 0 {
            0.0
        } else {
            wins as f64 / total_games as f64 * 100.0
        }
    }

    pub fn gems_spent_total(&self) -> u32 {
        self.drafts_played.iter().fold(0, |sum, draft| {
            sum + draft.cost_in_gems() - draft.reward_gems()
        })
    }

    pub fn gems_cost_per_pack(&self) -> f64 {
        self.gems_spent_total() as f64 / self.mythics_and_rares_from_drafts() as f64
    }

    pub fn gold_spent_total(&self) -> u32 {
        self.drafts_played.iter().fold(0, |sum, draft| {
            sum + draft.cost_in_gold() - (draft.reward_gems() * 5) // since a pack is 1000 gold or 200 gem
        })
    }

    pub fn gold_cost_per_pack(&self) -> f64 {
        self.gold_spent_total() as f64 / self.mythics_and_rares_from_drafts() as f64
    }

    pub fn drafts_completed(&self) -> u32 {
        self.drafts_played.len() as u32
    }

    /// Returns the number of the collected rares and mythic cards pulled from drafting,
    /// biased with the reward packs from those drafts - as opening those packs guarantee one rare or a mythic card.
    pub fn mythics_and_rares_from_drafts(&self) -> u32 {
        let total_packs = (self.packs_per_draft() * self.drafts_completed() as f64).floor() as u32;
        self.drafts_played.iter().fold(0, |sum, draft| {
            sum + draft.rares_collected() + draft.mythics_collected()
        }) + total_packs
    }
}

impl DraftSummaryComponent {
    pub fn new<P>(database_path: P) -> DraftSummaryComponent
    where
        P: AsRef<Path>,
    {
        DraftSummaryComponent {
            database: MtgaDb::new(database_path),
            selected_set: String::from("dsk"),
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

        let rares_summary = container(column![
            container(text(format!("{} RARES", set_name)).size(18))
                .width(Length::Fill)
                .center_x(),
            row![
                container(text("Rare wildcards (R):").size(16))
                    .padding(5)
                    .width(Length::FillPortion(75))
                    .align_x(Horizontal::Right),
                container(text(model.rare_wc_owned().to_string()).size(16))
                    .padding(5)
                    .width(Length::FillPortion(25))
                    .align_x(Horizontal::Center),
            ],
            row![
                container(text(format!("{} Packs Owned (P):", set_name)).size(16))
                    .padding(5)
                    .width(Length::FillPortion(75))
                    .align_x(Horizontal::Right),
                container(text(model.packs_owned().to_string()).size(16))
                    .padding(5)
                    .width(Length::FillPortion(25))
                    .align_x(Horizontal::Center),
            ],
            row![
                container(text(format!("{} Rares Owned (R):", set_name)).size(16))
                    .padding(5)
                    .width(Length::FillPortion(75))
                    .align_x(Horizontal::Right),
                container(text(model.rares_owned().to_string()).size(16))
                    .padding(5)
                    .width(Length::FillPortion(25))
                    .align_x(Horizontal::Center),
            ],
            row![
                container(text("Rares per Draft (N):").size(16))
                    .padding(5)
                    .width(Length::FillPortion(75))
                    .align_x(Horizontal::Right),
                container(text(format!("{:.2}", model.rares_per_draft())).size(16))
                    .padding(5)
                    .width(Length::FillPortion(25))
                    .align_x(Horizontal::Center),
            ],
            row![
                container(text("Packs per Draft (W):").size(16))
                    .padding(5)
                    .width(Length::FillPortion(75))
                    .align_x(Horizontal::Right),
                container(text(format!("{:.2}", model.packs_per_draft())).size(16))
                    .padding(5)
                    .width(Length::FillPortion(25))
                    .align_x(Horizontal::Center),
            ],
            row![
                container(text("Critical Point (drafts):").size(16))
                    .padding(5)
                    .width(Length::FillPortion(75))
                    .align_x(Horizontal::Right),
                container(text(format!("{:.2}", model.rares_critical_point())).size(16))
                    .padding(5)
                    .width(Length::FillPortion(25))
                    .align_x(Horizontal::Center),
            ],
            vertical_space().height(25),
            row![
                container(text("Rares needed:").size(16))
                    .padding(5)
                    .width(Length::FillPortion(75))
                    .align_x(Horizontal::Right),
                container(text(model.rares_needed().to_string()).size(16))
                    .padding(5)
                    .width(Length::FillPortion(25))
                    .align_x(Horizontal::Center),
            ],
        ])
        .style(ContainerStyle::SummaryPanel)
        .width(Length::Fixed(285.0));

        let mythics_summary = container(column![
            container(text(format!("{} MYTHICS", set_name)).size(18))
                .width(Length::Fill)
                .center_x(),
            row![
                container(text("Mythic wildcards (M):").size(16))
                    .padding(5)
                    .width(Length::FillPortion(75))
                    .align_x(Horizontal::Right),
                container(text(model.mythic_wc_owned().to_string()).size(16))
                    .padding(5)
                    .width(Length::FillPortion(25))
                    .align_x(Horizontal::Center),
            ],
            row![
                container(text(format!("{} Packs Owned (P):", set_name)).size(16))
                    .padding(5)
                    .width(Length::FillPortion(75))
                    .align_x(Horizontal::Right),
                container(text(model.packs_owned().to_string()).size(16))
                    .padding(5)
                    .width(Length::FillPortion(25))
                    .align_x(Horizontal::Center),
            ],
            row![
                container(text(format!("{} Mythics Owned (M):", set_name)).size(16))
                    .padding(5)
                    .width(Length::FillPortion(75))
                    .align_x(Horizontal::Right),
                container(text(model.mythics_owned().to_string()).size(16))
                    .padding(5)
                    .width(Length::FillPortion(25))
                    .align_x(Horizontal::Center),
            ],
            row![
                container(text("Mythics per Draft (N):").size(16))
                    .padding(5)
                    .width(Length::FillPortion(75))
                    .align_x(Horizontal::Right),
                container(text(format!("{:.2}", model.mythics_per_draft())).size(16))
                    .padding(5)
                    .width(Length::FillPortion(25))
                    .align_x(Horizontal::Center),
            ],
            row![
                container(text("Packs per Draft (W):").size(16))
                    .padding(5)
                    .width(Length::FillPortion(75))
                    .align_x(Horizontal::Right),
                container(text(format!("{:.2}", model.packs_per_draft())).size(16))
                    .padding(5)
                    .width(Length::FillPortion(25))
                    .align_x(Horizontal::Center),
            ],
            row![
                container(text("Critical Point (drafts):").size(16))
                    .padding(5)
                    .width(Length::FillPortion(75))
                    .align_x(Horizontal::Right),
                container(text(format!("{:.2}", model.mythics_critical_point())).size(16))
                    .padding(5)
                    .width(Length::FillPortion(25))
                    .align_x(Horizontal::Center),
            ],
            vertical_space().height(25),
            row![
                container(text("Mythics needed:").size(16))
                    .padding(5)
                    .width(Length::FillPortion(75))
                    .align_x(Horizontal::Right),
                container(text(model.mythics_needed().to_string()).size(16))
                    .padding(5)
                    .width(Length::FillPortion(25))
                    .align_x(Horizontal::Center),
            ],
        ])
        .style(ContainerStyle::SummaryPanel)
        .width(Length::Fixed(285.0));

        let general_stats = container(column![
            container(text(format!("{} STATS", set_name)).size(18))
                .width(Length::Fill)
                .center_x(),
            row![
                container(text("Avg Wins:").size(16))
                    .padding(5)
                    .width(Length::FillPortion(65))
                    .align_x(Horizontal::Right),
                container(
                    text(format!(
                        "{:.2} ({:.2})",
                        model.avg_wins().0,
                        model.avg_wins().1
                    ))
                    .size(16),
                )
                .padding(5)
                .width(Length::FillPortion(35))
                .align_x(Horizontal::Center),
                horizontal_space().width(50),
                container(text("Total Spent (gems):").size(16))
                    .padding(5)
                    .width(Length::FillPortion(65))
                    .align_x(Horizontal::Right),
                container(text(model.gems_spent_total().to_string()).size(16))
                    .padding(5)
                    .width(Length::FillPortion(35))
                    .align_x(Horizontal::Center),
            ],
            row![
                container(text("Win %:").size(16))
                    .padding(5)
                    .width(Length::FillPortion(65))
                    .align_x(Horizontal::Right),
                container(text(format!("{:.2}", model.win_percentage())).size(16))
                    .padding(5)
                    .width(Length::FillPortion(35))
                    .align_x(Horizontal::Center),
                horizontal_space().width(50),
                container(text("Cost per Pack (gems):").size(16))
                    .padding(5)
                    .width(Length::FillPortion(65))
                    .align_x(Horizontal::Right),
                container(text(format!("{:.2}", model.gems_cost_per_pack())).size(16))
                    .padding(5)
                    .width(Length::FillPortion(35))
                    .align_x(Horizontal::Center),
            ],
            row![
                container(text("Drafts completed:").size(16))
                    .padding(5)
                    .width(Length::FillPortion(65))
                    .align_x(Horizontal::Right),
                container(text(model.drafts_completed().to_string()).size(16))
                    .padding(5)
                    .width(Length::FillPortion(35))
                    .align_x(Horizontal::Center),
                horizontal_space().width(50),
                container(text("Total Spent (gold):").size(16))
                    .padding(5)
                    .width(Length::FillPortion(65))
                    .align_x(Horizontal::Right),
                container(text(model.gold_spent_total().to_string()).size(16))
                    .padding(5)
                    .width(Length::FillPortion(35))
                    .align_x(Horizontal::Center),
            ],
            row![
                container(text("Mythics & Rares from drafts:").size(16))
                    .padding(5)
                    .width(Length::FillPortion(65))
                    .align_x(Horizontal::Right),
                container(text(model.mythics_and_rares_from_drafts().to_string()).size(16),)
                    .padding(5)
                    .width(Length::FillPortion(35))
                    .align_x(Horizontal::Center),
                horizontal_space().width(50),
                container(text("Cost per Pack (gold):").size(16))
                    .padding(5)
                    .width(Length::FillPortion(65))
                    .align_x(Horizontal::Right),
                container(text(format!("{:.2}", model.gold_cost_per_pack())).size(16))
                    .padding(5)
                    .width(Length::FillPortion(35))
                    .align_x(Horizontal::Center),
            ]
        ])
        .style(ContainerStyle::SummaryPanel)
        .width(Length::Fixed(620.0));

        let draft_summary = container(
            column![
                row![rares_summary, mythics_summary].spacing(50),
                general_stats
            ]
            .spacing(20),
        )
        .padding(20);

        let draft_panel_content = column![draft_summary];
        container(draft_panel_content).into()
    }
}
