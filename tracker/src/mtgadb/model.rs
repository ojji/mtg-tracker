//! The main model types are `AccountInfoResult`, `InventoryUpdateResult` and `CollectionResult`, appearing as the main
//! JSON objects in the log file. Every main model type has a `Timestamp` and an `Attachment` field, describing when the
//! event occured and containing the detailed event object.
use crate::Result;
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;

use chrono::{DateTime, NaiveDateTime};
use rusqlite::{types::FromSql, Connection, ToSql};
use serde::{de::Visitor, Deserialize, Serialize};

pub trait ParseResult {
    fn get_prefix(&self) -> &str;
    fn get_content(&self) -> &str;
    fn get_date(&self) -> &Option<NaiveDateTime>;
    fn set_common_fields(&mut self, prefix: String, content: String, date: Option<NaiveDateTime>);
}

#[derive(Debug, Clone)]
pub enum ParseResults {
    UnknownResult(UnknownResult),
    SceneChangeResult(SceneChangeResult),
    AccountInfoResult(AccountInfoResult),
    InventoryUpdateResult(InventoryUpdateResult),
    InventoryResult(InventoryResult),
    CollectionResult(CollectionResult),
}

impl ParseResult for ParseResults {
    fn get_prefix(&self) -> &str {
        match self {
            ParseResults::UnknownResult(r) => r.get_prefix(),
            ParseResults::SceneChangeResult(r) => r.get_prefix(),
            ParseResults::AccountInfoResult(r) => r.get_prefix(),
            ParseResults::InventoryUpdateResult(r) => r.get_prefix(),
            ParseResults::InventoryResult(r) => r.get_prefix(),
            ParseResults::CollectionResult(r) => r.get_prefix(),
        }
    }

    fn get_content(&self) -> &str {
        match self {
            ParseResults::UnknownResult(r) => r.get_content(),
            ParseResults::SceneChangeResult(r) => r.get_content(),
            ParseResults::AccountInfoResult(r) => r.get_content(),
            ParseResults::InventoryUpdateResult(r) => r.get_content(),
            ParseResults::InventoryResult(r) => r.get_content(),
            ParseResults::CollectionResult(r) => r.get_content(),
        }
    }

    fn get_date(&self) -> &Option<NaiveDateTime> {
        match self {
            ParseResults::UnknownResult(r) => r.get_date(),
            ParseResults::SceneChangeResult(r) => r.get_date(),
            ParseResults::AccountInfoResult(r) => r.get_date(),
            ParseResults::InventoryUpdateResult(r) => r.get_date(),
            ParseResults::InventoryResult(r) => r.get_date(),
            ParseResults::CollectionResult(r) => r.get_date(),
        }
    }

    fn set_common_fields(&mut self, prefix: String, content: String, date: Option<NaiveDateTime>) {
        match self {
            ParseResults::UnknownResult(r) => r.set_common_fields(prefix, content, date),
            ParseResults::SceneChangeResult(r) => r.set_common_fields(prefix, content, date),
            ParseResults::AccountInfoResult(r) => r.set_common_fields(prefix, content, date),
            ParseResults::InventoryUpdateResult(r) => r.set_common_fields(prefix, content, date),
            ParseResults::InventoryResult(r) => r.set_common_fields(prefix, content, date),
            ParseResults::CollectionResult(r) => r.set_common_fields(prefix, content, date),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Hash, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct CollectionResult {
    timestamp: String,
    attachment: Vec<CollectedCard>,
    #[serde(skip)]
    prefix: String,
    #[serde(skip)]
    date: Option<NaiveDateTime>,
    #[serde(skip)]
    content: String,
}

impl CollectionResult {
    pub fn timestamp_str(&self) -> &str {
        &self.timestamp
    }

    pub fn friendly_time(&self) -> String {
        match DateTime::parse_from_rfc3339(self.timestamp_str()) {
            Ok(time) => time.format("%Y-%m-%d %H:%M:%S").to_string(),
            Err(e) => format!("time conversion err: {}", e),
        }
    }

    pub fn payload(&self) -> &Vec<CollectedCard> {
        &self.attachment
    }
}

impl ParseResult for CollectionResult {
    fn get_prefix(&self) -> &str {
        &self.prefix
    }

    fn get_content(&self) -> &str {
        &self.content
    }

    fn get_date(&self) -> &Option<NaiveDateTime> {
        &self.date
    }

    fn set_common_fields(&mut self, prefix: String, content: String, date: Option<NaiveDateTime>) {
        self.prefix = prefix;
        self.date = date;
        self.content = content;
    }
}

#[derive(Debug, Serialize, Deserialize, Hash, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct InventoryResult {
    timestamp: String,
    attachment: PlayerInventoryData,
    #[serde(skip)]
    prefix: String,
    #[serde(skip)]
    date: Option<NaiveDateTime>,
    #[serde(skip)]
    content: String,
}

impl InventoryResult {
    pub fn timestamp_str(&self) -> &str {
        &self.timestamp
    }

    pub fn friendly_time(&self) -> String {
        match DateTime::parse_from_rfc3339(self.timestamp_str()) {
            Ok(time) => time.format("%Y-%m-%d %H:%M:%S").to_string(),
            Err(e) => format!("time conversion err: {}", e),
        }
    }

    pub fn payload(&self) -> &PlayerInventoryData {
        &self.attachment
    }
}

impl ParseResult for InventoryResult {
    fn get_prefix(&self) -> &str {
        &self.prefix
    }

    fn get_content(&self) -> &str {
        &self.content
    }

    fn get_date(&self) -> &Option<NaiveDateTime> {
        &self.date
    }

    fn set_common_fields(&mut self, prefix: String, content: String, date: Option<NaiveDateTime>) {
        self.prefix = prefix;
        self.date = date;
        self.content = content;
    }
}

#[derive(Debug, Serialize, Deserialize, Hash, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct InventoryUpdateResult {
    timestamp: String,
    source: String,
    attachment: InventoryUpdateData,
    #[serde(skip)]
    prefix: String,
    #[serde(skip)]
    date: Option<NaiveDateTime>,
    #[serde(skip)]
    content: String,
}

impl InventoryUpdateResult {
    pub fn friendly_time(&self) -> String {
        match DateTime::parse_from_rfc3339(self.timestamp_str()) {
            Ok(time) => time.format("%Y-%m-%d %H:%M:%S").to_string(),
            Err(e) => format!("time conversion err: {}", e),
        }
    }

    pub fn timestamp_str(&self) -> &str {
        &self.timestamp
    }

    pub fn update_source(&self) -> &str {
        &self.source
    }

    pub fn source_context(&self) -> &str {
        &self.attachment.context.source
    }

    pub fn payload(&self) -> &InventoryUpdateData {
        &self.attachment
    }

    pub fn mythic_wildcard_delta(&self) -> i32 {
        self.attachment.delta.wc_mythic_delta
    }

    pub fn rare_wildcard_delta(&self) -> i32 {
        self.attachment.delta.wc_rare_delta
    }

    pub fn uncommon_wildcard_delta(&self) -> i32 {
        self.attachment.delta.wc_uncommon_delta
    }

    pub fn common_wildcard_delta(&self) -> i32 {
        self.attachment.delta.wc_common_delta
    }

    pub fn xp_gained(&self) -> i32 {
        self.attachment.xp_gained
    }

    pub fn gems_delta(&self) -> i32 {
        self.attachment.delta.gems_delta
    }

    pub fn gold_delta(&self) -> i32 {
        self.attachment.delta.gold_delta
    }

    pub fn draft_token_delta(&self) -> Option<i32> {
        if self
            .attachment
            .delta
            .custom_token_delta
            .iter()
            .any(|token| token.is_draft_token())
        {
            let sum = self
                .attachment
                .delta
                .custom_token_delta
                .iter()
                .filter_map(|token| {
                    if token.is_draft_token() {
                        Some(token.delta)
                    } else {
                        None
                    }
                })
                .sum::<i32>();

            Some(sum)
        } else {
            None
        }
    }

    pub fn orbs_delta(&self) -> Option<i32> {
        if self
            .attachment
            .delta
            .custom_token_delta
            .iter()
            .any(|token| token.is_orb_token())
        {
            let sum = self
                .attachment
                .delta
                .custom_token_delta
                .iter()
                .filter_map(|token| {
                    if token.is_orb_token() {
                        Some(token.delta)
                    } else {
                        None
                    }
                })
                .sum::<i32>();

            Some(sum)
        } else {
            None
        }
    }

    pub fn vault_delta_percent(&self) -> f32 {
        let cards_vault_sum = self
            .attachment
            .aetherized_cards
            .iter()
            .filter_map(|card| {
                if !card.added_to_inventory {
                    Some(card.vault_progress.inner_value as f32)
                } else {
                    None
                }
            })
            .sum::<f32>();

        (self.attachment.delta.vault_progress_delta.inner_value as f32 + cards_vault_sum) / 10.0
    }

    pub fn cards_added(&self) -> Vec<u32> {
        self.attachment
            .aetherized_cards
            .iter()
            .filter_map(|card| {
                if card.added_to_inventory && card.is_actual_card() {
                    Some(card.grp_id)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn tickets(&self) -> Option<&Vec<TicketStack>> {
        self.attachment.delta.tickets.as_ref()
    }

    pub fn packs_delta(&self) -> Option<&Vec<BoosterStack>> {
        if self.attachment.delta.booster_delta.is_empty() {
            None
        } else {
            Some(self.attachment.delta.booster_delta.as_ref())
        }
    }

    pub fn art_skins_added(&self) -> Option<&Vec<ArtSkin>> {
        if self.attachment.delta.art_skins_added.is_empty() {
            None
        } else {
            Some(self.attachment.delta.art_skins_added.as_ref())
        }
    }

    pub fn art_skins_removed(&self) -> Option<&Vec<ArtSkin>> {
        self.attachment.delta.art_skins_removed.as_ref()
    }

    pub fn vanity_items_added(&self) -> Option<&Vec<String>> {
        if self.attachment.delta.vanity_items_added.is_empty() {
            None
        } else {
            Some(self.attachment.delta.vanity_items_added.as_ref())
        }
    }

    pub fn vanity_items_removed(&self) -> Option<&Vec<String>> {
        self.attachment.delta.vanity_items_removed.as_ref()
    }
}

impl ParseResult for InventoryUpdateResult {
    fn get_prefix(&self) -> &str {
        &self.prefix
    }

    fn get_content(&self) -> &str {
        &self.content
    }

    fn get_date(&self) -> &Option<NaiveDateTime> {
        &self.date
    }

    fn set_common_fields(&mut self, prefix: String, content: String, date: Option<NaiveDateTime>) {
        self.prefix = prefix;
        self.date = date;
        self.content = content;
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct AccountInfoResult {
    timestamp: String,
    attachment: AccountInfoData,
    #[serde(skip)]
    prefix: String,
    #[serde(skip)]
    date: Option<NaiveDateTime>,
    #[serde(skip)]
    content: String,
}

impl AccountInfoResult {
    pub fn timestamp_str(&self) -> &str {
        &self.timestamp
    }

    pub fn screen_name(&self) -> &str {
        &self.attachment.screen_name
    }

    pub fn user_id(&self) -> &str {
        &self.attachment.user_id
    }
}

impl ParseResult for AccountInfoResult {
    fn get_prefix(&self) -> &str {
        &self.prefix
    }

    fn get_content(&self) -> &str {
        &self.content
    }

    fn get_date(&self) -> &Option<NaiveDateTime> {
        &self.date
    }

    fn set_common_fields(&mut self, prefix: String, content: String, date: Option<NaiveDateTime>) {
        self.prefix = prefix;
        self.date = date;
        self.content = content;
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfoData {
    user_id: String,
    screen_name: String,
}

#[derive(Debug, Clone)]
pub struct UnknownResult {
    prefix: String,
    content: String,
    date: Option<NaiveDateTime>,
}

impl UnknownResult {
    pub fn new(prefix: String, date: Option<NaiveDateTime>, content: String) -> UnknownResult {
        UnknownResult {
            prefix,
            content,
            date,
        }
    }
}

impl ParseResult for UnknownResult {
    fn get_prefix(&self) -> &str {
        &self.prefix
    }

    fn get_content(&self) -> &str {
        &self.content
    }

    fn get_date(&self) -> &Option<NaiveDateTime> {
        &self.date
    }

    fn set_common_fields(&mut self, prefix: String, content: String, date: Option<NaiveDateTime>) {
        self.prefix = prefix;
        self.content = content;
        self.date = date;
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SceneChangeResult {
    from_scene_name: String,
    to_scene_name: String,
    initiator: String,
    context: Option<String>,
    #[serde(skip)]
    prefix: String,
    #[serde(skip)]
    date: Option<NaiveDateTime>,
    #[serde(skip)]
    content: String,
}

impl SceneChangeResult {
    pub fn from_scene(&self) -> &str {
        &self.from_scene_name
    }

    pub fn to_scene(&self) -> &str {
        &self.to_scene_name
    }

    pub fn initiator(&self) -> &str {
        &self.initiator
    }

    pub fn context(&self) -> Option<&str> {
        self.context.as_deref()
    }
}

impl ParseResult for SceneChangeResult {
    fn get_prefix(&self) -> &str {
        &self.prefix
    }

    fn get_content(&self) -> &str {
        &self.content
    }

    fn get_date(&self) -> &Option<NaiveDateTime> {
        &self.date
    }

    fn set_common_fields(&mut self, prefix: String, content: String, date: Option<NaiveDateTime>) {
        self.prefix = prefix;
        self.date = date;
        self.content = content;
    }
}

/// # Card Objects
/// Card objects represent individual Magic: The Gathering cards that players could obtain and add to their collection
/// (with a few minor exceptions).
///
/// Cards are the API’s most complex object. You are encouraged to thoroughly read this document and also the
/// article about layouts and images.
///
/// ## Card names
/// Internally, Scryfall tracks the uniqueness of “Oracle names.” (i.e. names you can pick when an effect asks
/// you to “choose a card name”). Each unique Oracle name is separately available in the [card names catalog](https://scryfall.com/docs/api/catalogs/card-names).
///
/// Note that while most Oracle card names are unique, Scryfall also indexes other objects such as tokens
/// and Unstable set variants which do not always have a unique name.
///
/// ## Multiface Cards
/// Magic cards can have multiple faces or multiple cards printed on one card stock. The faces could be shown divided
/// on the front of the card as in split cards and [flip cards](https://scryfall.com/search?q=is%3Asplit), or the card
/// can be double-faced as in [transform cards](https://scryfall.com/search?q=is%3Atransform) and
/// [modal DFCs](https://scryfall.com/search?q=is%3Amdfc).
///
/// Scryfall represents multi-face cards as a single object with a `card_faces` array describing the distinct faces.
///
/// ## Example Json object:
/// ```
/// {
///    "object": "card",
///    "id": "0000579f-7b35-4ed3-b44c-db2a538066fe",
///    "oracle_id": "44623693-51d6-49ad-8cd7-140505caf02f",
///    "multiverse_ids": [
///      109722
///    ],
///    "mtgo_id": 25527,
///    "mtgo_foil_id": 25528,
///    "tcgplayer_id": 14240,
///    "cardmarket_id": 13850,
///    "name": "Fury Sliver",
///    "lang": "en",
///    "released_at": "2006-10-06",
///    "uri": "https://api.scryfall.com/cards/0000579f-7b35-4ed3-b44c-db2a538066fe",
///    "scryfall_uri": "https://scryfall.com/card/tsp/157/fury-sliver?utm_source=api",
///    "layout": "normal",
///    "highres_image": true,
///    "image_status": "highres_scan",
///    "image_uris": {
///      "small": "https://c1.scryfall.com/file/scryfall-cards/small/front/0/0/0000579f-7b35-4ed3-b44c-db2a538066fe.jpg?1562894979",
///      "normal": "https://c1.scryfall.com/file/scryfall-cards/normal/front/0/0/0000579f-7b35-4ed3-b44c-db2a538066fe.jpg?1562894979",
///      "large": "https://c1.scryfall.com/file/scryfall-cards/large/front/0/0/0000579f-7b35-4ed3-b44c-db2a538066fe.jpg?1562894979",
///      "png": "https://c1.scryfall.com/file/scryfall-cards/png/front/0/0/0000579f-7b35-4ed3-b44c-db2a538066fe.png?1562894979",
///      "art_crop": "https://c1.scryfall.com/file/scryfall-cards/art_crop/front/0/0/0000579f-7b35-4ed3-b44c-db2a538066fe.jpg?1562894979",
///      "border_crop": "https://c1.scryfall.com/file/scryfall-cards/border_crop/front/0/0/0000579f-7b35-4ed3-b44c-db2a538066fe.jpg?1562894979"
///    },
///    "mana_cost": "{5}{R}",
///    "cmc": 6,
///    "type_line": "Creature — Sliver",
///    "oracle_text": "All Sliver creatures have double strike.",
///    "power": "3",
///    "toughness": "3",
///    "colors": [
///      "R"
///    ],
///    "color_identity": [
///      "R"
///    ],
///    "keywords": [],
///    "legalities": {
///      "standard": "not_legal",
///      "future": "not_legal",
///      "historic": "not_legal",
///      "gladiator": "not_legal",
///      "pioneer": "not_legal",
///      "explorer": "not_legal",
///      "modern": "legal",
///      "legacy": "legal",
///      "pauper": "not_legal",
///      "vintage": "legal",
///      "penny": "legal",
///      "commander": "legal",
///      "brawl": "not_legal",
///      "historicbrawl": "not_legal",
///      "alchemy": "not_legal",
///      "paupercommander": "restricted",
///      "duel": "legal",
///      "oldschool": "not_legal",
///      "premodern": "not_legal"
///    },
///    "games": [
///      "paper",
///      "mtgo"
///    ],
///    "reserved": false,
///    "foil": true,
///    "nonfoil": true,
///    "finishes": [
///      "nonfoil",
///      "foil"
///    ],
///    "oversized": false,
///    "promo": false,
///    "reprint": false,
///    "variation": false,
///    "set_id": "c1d109bc-ffd8-428f-8d7d-3f8d7e648046",
///    "set": "tsp",
///    "set_name": "Time Spiral",
///    "set_type": "expansion",
///    "set_uri": "https://api.scryfall.com/sets/c1d109bc-ffd8-428f-8d7d-3f8d7e648046",
///    "set_search_uri": "https://api.scryfall.com/cards/search?order=set\\u0026q=e%3Atsp\\u0026unique=prints",
///    "scryfall_set_uri": "https://scryfall.com/sets/tsp?utm_source=api",
///    "rulings_uri": "https://api.scryfall.com/cards/0000579f-7b35-4ed3-b44c-db2a538066fe/rulings",
///    "prints_search_uri": "https://api.scryfall.com/cards/search?order=released\\u0026q=oracleid%3A44623693-51d6-49ad-8cd7-140505caf02f\\u0026unique=prints",
///    "collector_number": "157",
///    "digital": false,
///    "rarity": "uncommon",
///    "flavor_text": "\"A rift opened, and our arrows were abruptly stilled. To move was to push the world. But the sliver's claw still twitched, red wounds appeared in Thed's chest, and ribbons of blood hung in the air.\"\n—Adom Capashen, Benalish hero",
///    "card_back_id": "0aeebaf5-8c7d-4636-9e82-8c27447861f7",
///    "artist": "Paolo Parente",
///    "artist_ids": [
///      "d48dd097-720d-476a-8722-6a02854ae28b"
///    ],
///    "illustration_id": "2fcca987-364c-4738-a75b-099d8a26d614",
///    "border_color": "black",
///    "frame": "2003",
///    "full_art": false,
///    "textless": false,
///    "booster": true,
///    "story_spotlight": false,
///    "edhrec_rank": 5602,
///    "penny_rank": 10384,
///    "prices": {
///      "usd": "0.29",
///      "usd_foil": "4.50",
///      "usd_etched": null,
///      "eur": "0.13",
///      "eur_foil": "0.49",
///      "tix": "0.02"
///    },
///    "related_uris": {
///      "gatherer": "https://gatherer.wizards.com/Pages/Card/Details.aspx?multiverseid=109722",
///      "tcgplayer_infinite_articles": "https://infinite.tcgplayer.com/search?contentMode=article\\u0026game=magic\\u0026partner=scryfall\\u0026q=Fury+Sliver\\u0026utm_campaign=affiliate\\u0026utm_medium=api\\u0026utm_source=scryfall",
///      "tcgplayer_infinite_decks": "https://infinite.tcgplayer.com/search?contentMode=deck\\u0026game=magic\\u0026partner=scryfall\\u0026q=Fury+Sliver\\u0026utm_campaign=affiliate\\u0026utm_medium=api\\u0026utm_source=scryfall",
///      "edhrec": "https://edhrec.com/route/?cc=Fury+Sliver"
///    }
/// }
/// ```

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ScryCard {
    /// This card’s Arena ID, if any. A large percentage of cards are not available on Arena and do not have this ID.
    arena_id: Option<u32>,

    /// A unique ID for this card in Scryfall’s database.
    id: Guid,

    /// A [language](https://scryfall.com/docs/api/languages) code for this printing.
    lang: String,

    /// This card’s Magic Online ID (also known as the Catalog ID), if any.
    /// A large percentage of cards are not available on Magic Online and do not have this ID.
    mtgo_id: Option<u32>,

    /// This card’s foil Magic Online ID (also known as the Catalog ID), if any.
    /// A large percentage of cards are not available on Magic Online and do not have this ID.
    mtgo_foil_id: Option<u32>,

    /// This card’s multiverse IDs on Gatherer, if any, as an array of integers. Note that Scryfall includes many
    /// promo cards, tokens, and other esoteric objects that do not have these identifiers.
    multiverse_ids: Option<Vec<u32>>,

    /// This card’s ID on [TCGplayer’s API](https://docs.tcgplayer.com/docs), also known as the `productId`.
    tcgplayer_id: Option<u32>,

    /// This card’s ID on [TCGplayer’s API](https://docs.tcgplayer.com/docs), for its etched version if that version
    /// is a separate product.
    tcgplayer_etched_id: Option<u32>,

    /// This card’s ID on Cardmarket’s API, also known as the `idProduct`.
    cardmarket_id: Option<u32>,

    /// A content type for this object, always `card`.
    object: String,

    /// A unique ID for this card’s oracle identity. This value is consistent across reprinted card editions, and unique
    /// among different cards with the same name (tokens, Unstable variants, etc).
    oracle_id: Option<Guid>,

    /// A link to where you can begin paginating all re/prints for this card on Scryfall’s API.
    prints_search_uri: Uri,

    /// A link to this card’s [rulings list](https://scryfall.com/docs/api/rulings) on Scryfall’s API.
    rulings_uri: Uri,

    /// A link to this card’s permapage on Scryfall’s website.
    scryfall_uri: Uri,

    /// A link to this card object on Scryfall’s API.
    uri: Uri,

    /// If this card is closely related to other cards, this property will be an array with Related Card Objects.
    all_parts: Option<Vec<ScryRelatedCard>>,

    /// An array of Card Face objects, if this card is multifaced.
    card_faces: Option<Vec<ScryCardFace>>,

    /// The card’s converted mana cost. Note that some funny cards have fractional mana costs.
    cmc: Option<f32>,

    /// This card’s color identity.
    color_identity: Colors,

    /// The colors in this card’s color indicator, if any. A null value for this field indicates the card does not have
    /// one.
    color_indicator: Option<Colors>,

    /// This card’s colors, if the overall card has colors defined by the rules. Otherwise the colors will be on
    /// the card_faces objects, see below.
    colors: Option<Colors>,

    /// This card’s overall rank/popularity on EDHREC. Not all cards are ranked.
    edhrec_rank: Option<u32>,

    /// This card’s hand modifier, if it is Vanguard card. This value will contain a delta, such as `-1`.
    hand_modifier: Option<String>,

    /// An array of keywords that this card uses, such as `'Flying'` and `'Cumulative upkeep'`.
    keywords: Vec<String>,

    /// A code for this card’s [layout](https://scryfall.com/docs/api/layouts).
    layout: String,

    /// An object describing the legality of this card across play formats. Possible legalities are `legal`,
    /// `not_legal`, `restricted`, and `banned`.
    legalities: HashMap<String, Legality>,

    /// This card’s life modifier, if it is Vanguard card. This value will contain a delta, such as `+2`.
    life_modifier: Option<String>,

    /// This loyalty if any. Note that some cards have loyalties that are not numeric, such as `X`.
    loyalty: Option<String>,

    /// The mana cost for this card. This value will be any empty string `""` if the cost is absent. Remember that per
    /// the game rules, a missing mana cost and a mana cost of `{0}` are different values.
    /// Multi-faced cards will report this value in card faces.
    mana_cost: Option<String>,

    /// The name of this card. If this card has multiple faces, this field will contain both names separated by `␣//␣`.
    name: String,

    /// The Oracle text for this card, if any.
    oracle_text: Option<String>,

    /// True if this card is oversized.
    oversized: bool,

    /// This card’s rank/popularity on Penny Dreadful. Not all cards are ranked.
    penny_rank: Option<u32>,

    /// This card’s power, if any. Note that some cards have powers that are not numeric, such as `*`.
    power: Option<String>,

    /// Colors of mana that this card could produce.
    produced_mana: Option<Colors>,

    /// True if this card is on the Reserved List.
    reserved: bool,

    /// This card’s toughness, if any. Note that some cards have toughnesses that are not numeric, such as `*`.
    toughness: Option<String>,

    /// The type line of this card.
    type_line: Option<String>,

    /// The name of the illustrator of this card. Newly spoiled cards may not have this field yet.
    artist: Option<String>,

    /// Whether this card is found in boosters.
    booster: bool,

    /// This card’s border color: `black`, `white`, `borderless`, `silver`, or `gold`.
    border_color: String,

    /// The Scryfall ID for the card back design present on this card.
    card_back_id: Option<Guid>,

    /// This card’s collector number. Note that collector numbers can contain non-numeric characters,
    /// such as letters or `★`.
    collector_number: String,

    /// True if you should consider [avoiding use of this print](https://scryfall.com/blog/regarding-wotc-s-recent-statement-on-depictions-of-racism-220) downstream.
    content_warning: Option<bool>,

    /// True if this card was only released in a video game.
    digital: bool,

    /// An array of computer-readable flags that indicate if this card can come in `foil`, `nonfoil`, `etched`,
    /// or `glossy` finishes.
    finishes: Vec<String>,

    /// The just-for-fun name printed on the card (such as for Godzilla series cards).
    flavor_name: Option<String>,

    /// The flavor text, if any.
    flavor_text: Option<String>,

    /// This card’s [frame effects](https://scryfall.com/docs/api/layouts), if any.
    frame_effects: Option<Vec<String>>,

    /// This card’s [frame layout](https://scryfall.com/docs/api/layouts).
    frame: String,

    /// True if this card’s artwork is larger than normal.
    full_art: bool,

    /// A list of games that this card print is available in, `paper`, `arena`, and/or `mtgo`.
    games: Vec<String>,

    /// True if this card’s imagery is high resolution.
    highres_image: bool,

    /// A unique identifier for the card artwork that remains consistent across reprints. Newly spoiled cards may not have this field yet.
    illustration_id: Option<Guid>,

    /// A computer-readable indicator for the state of this card’s image, one of `missing`, `placeholder`, `lowres`,
    /// or `highres_scan`.
    image_status: String,

    /// An object listing available imagery for this card. See the [Card Imagery](https://scryfall.com/docs/api/images) article for more information.
    image_uris: Option<HashMap<String, Uri>>,

    /// An object containing daily price information for this card, including `usd`, `usd_foil`, `usd_etched`, `eur`,
    /// and `tix` prices, as strings.
    prices: HashMap<String, Option<String>>,

    /// The localized name printed on this card, if any.
    printed_name: Option<String>,

    /// The localized text printed on this card, if any.
    printed_text: Option<String>,

    /// The localized type line printed on this card, if any.
    printed_type_line: Option<String>,

    /// True if this card is a promotional print.
    promo: bool,

    /// An array of strings describing what categories of promo cards this card falls into.
    promo_types: Option<Vec<String>>,

    // purchase_uris: Object 	/// An object providing URIs to this card’s listing on major marketplaces. (seems missing)
    /// This card’s rarity. One of `common`, `uncommon`, `rare`, `special`, `mythic`, or `bonus`.
    rarity: String,

    /// An object providing URIs to this card’s listing on other Magic: The Gathering online resources.
    related_uris: HashMap<String, Uri>,

    /// The date this card was first released.
    released_at: Date,

    /// True if this card is a reprint.
    reprint: bool,

    /// A link to this card’s set on Scryfall’s website.
    scryfall_set_uri: Uri,

    /// This card’s full set name.
    set_name: String,

    /// A link to where you can begin paginating this card’s set on the Scryfall API.
    set_search_uri: Uri,

    /// The type of set this printing is in.
    set_type: String,

    /// A link to this card’s [set object](https://scryfall.com/docs/api/sets) on Scryfall’s API.
    set_uri: Uri,

    /// This card’s set code.
    set: String,

    /// This card’s Set object UUID.
    set_id: Guid,

    /// True if this card is a Story Spotlight.
    story_spotlight: bool,

    /// True if the card is printed without text.
    textless: bool,

    /// Whether this card is a variation of another printing.
    variation: bool,

    /// The printing ID of the printing this card is a variation of.
    variation_of: Option<Guid>,

    /// The security stamp on this card, if any. One of `oval`, `triangle`, `acorn`, `arena`, or `heart`.
    security_stamp: Option<String>,

    /// This card’s watermark, if any.
    watermark: Option<String>,

    /// The date this card was previewed, a link to the preview for this card, the name of the source that previewed
    /// this card.
    preview: Option<HashMap<String, String>>,
}

impl ScryCard {
    pub fn arena_id(&self) -> Option<u32> {
        self.arena_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set(&self) -> &str {
        &self.set
    }

    pub fn rarity(&self) -> &str {
        &self.rarity
    }

    pub fn unreversed_lowercase_name(&self) -> String {
        if self.layout == "reversible_card" {
            let separator = self.name.find('/').unwrap();
            self.name[0..separator - 1].to_lowercase()
        } else {
            self.name.to_lowercase()
        }
    }

    pub fn is_available_in_arena(&self) -> bool {
        self.games.contains(&String::from("arena"))
    }

    pub fn lowercase_artist(&self) -> Option<String> {
        self.artist.as_ref().map(|artist| artist.to_lowercase())
    }

    pub fn booster(&self) -> bool {
        self.booster
    }

    pub fn collector_number(&self) -> &str {
        self.collector_number.as_str()
    }

    pub fn lang(&self) -> &str {
        &self.lang
    }

    pub fn scryfall_uri(&self) -> &Uri {
        &self.scryfall_uri
    }

    fn get_image_uri(&self) -> Uri {
        self.image_uris
            .as_ref()
            .unwrap_or(&HashMap::new())
            .get("normal")
            .unwrap_or(&Uri::new())
            .to_owned()
    }
}

impl Display for ScryCard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({}/{}) lang:{} ({}) (#{})",
            self.name,
            self.set,
            self.collector_number,
            self.lang,
            self.artist
                .as_ref()
                .unwrap_or(&String::from("unknown artist")),
            self.arena_id.unwrap_or_default()
        )
    }
}

/// # Related Card Objects
/// Cards that are closely related to other cards (because they call them by name, or generate a token, or meld, etc)
/// have a `all_parts` property that contains Related Card objects.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ScryRelatedCard {
    /// An unique ID for this card in Scryfall’s database.
    pub id: Guid,

    /// A content type for this object, always `related_card`.
    pub object: String,

    /// A field explaining what role this card plays in this relationship, one of `token`, `meld_part`, `meld_result`,
    /// or `combo_piece`.
    pub component: String,

    /// The name of this particular related card.
    pub name: String,

    /// The type line of this card.
    pub type_line: String,

    /// A URI where you can retrieve a full object describing this card on Scryfall’s API.
    pub uri: Uri,
}

/// # Card Face Objects
/// Multiface cards have a card_faces property containing at least two Card Face objects.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ScryCardFace {
    /// The name of the illustrator of this card face. Newly spoiled cards may not have this field yet.
    pub artist: Option<String>,

    /// The mana value of this particular face, if the card is reversible.
    pub cmc: Option<f32>,

    /// The colors in this face’s color indicator, if any.
    pub color_indicator: Option<Colors>,

    /// This face’s colors, if the game defines colors for the individual face of this card.
    pub colors: Option<Colors>,

    /// The flavor text printed on this face, if any.
    pub flavor_text: Option<String>,

    /// A unique identifier for the card face artwork that remains consistent across reprints.
    /// Newly spoiled cards may not have this field yet.
    pub illustration_id: Option<Guid>,

    /// An object providing URIs to imagery for this face, if this is a double-sided card.
    /// If this card is not double-sided, then the `image_uris` property will be part of the parent object instead.
    pub image_uris: Option<HashMap<String, Uri>>,

    /// The layout of this card face, if the card is reversible.
    pub layout: Option<String>,

    /// This face’s loyalty, if any.
    pub loyalty: Option<String>,

    /// The mana cost for this face. This value will be any empty string `""` if the cost is absent.
    /// Remember that per the game rules, a missing mana cost and a mana cost of `{0}` are different values.
    pub mana_cost: String,

    /// The name of this particular face.
    pub name: String,

    /// A content type for this object, always `card_face`.
    pub object: String,

    /// The Oracle ID of this particular face, if the card is reversible.
    pub oracle_id: Option<Guid>,

    /// The Oracle text for this face, if any.
    pub oracle_text: Option<String>,

    /// This face’s power, if any. Note that some cards have powers that are not numeric, such as `*`.
    pub power: Option<String>,

    /// The localized name printed on this face, if any.
    pub printed_name: Option<String>,

    /// The localized text printed on this face, if any.
    pub printed_text: Option<String>,

    /// The localized type line printed on this face, if any.
    pub printed_type_line: Option<String>,

    /// This face’s toughness, if any.
    pub toughness: Option<String>,

    /// The type line of this particular face, if the card is reversible.
    pub type_line: Option<String>,

    /// The watermark on this particulary card face, if any.
    pub watermark: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Hash)]
pub struct Guid(String);

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct Uri(String);

impl Uri {
    pub fn new() -> Uri {
        Uri(String::from(""))
    }

    pub fn from(s: &str) -> Uri {
        let mut uri = Uri::new();
        uri.0.push_str(s);
        uri
    }
}

impl ToSql for Uri {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        self.0.to_sql()
    }
}

impl FromSql for Uri {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        Ok(Uri::from(value.as_str()?))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Colors(Vec<Color>);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Color(String);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Legality(String);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Date(String);

/// `PlayerInventoryData` contains what the player has in their inventory - wildcards, booster packs, draft tokens etc.
///
/// Example object in the json body:
/// ```
/// {
///   "wcCommon": 38,
///   "wcUncommon": 71,
///   "wcRare": 45,
///   "wcMythic": 30,
///   "gold": 793775,
///   "gems": 25080,
///   "wcTrackPosition": 0,
///   "vaultProgress": 27.1,
///   "boosters": [
///     {
///       "collationId": 100023,
///       "count": 65
///     },
///     {
///       "collationId": 100024,
///       "count": 51
///     },
///     {
///       "collationId": 100025,
///       "count": 50
///     },
///     {
///       "collationId": 100026,
///       "count": 58
///     },
///     {
///       "collationId": 400026,
///       "count": 9
///     },
///     {
///       "collationId": 100027,
///       "count": 53
///     },
///     {
///       "collationId": 100028,
///       "count": 40
///     },
///     {
///       "collationId": 400028,
///       "count": 6
///     },
///     {
///       "collationId": 100029,
///       "count": 37
///     },
///     {
///       "collationId": 100030,
///       "count": 15
///     }
///   ],
///   "vouchers": [],
///   "basicLandSet": null,
///   "latestBasicLandSet": null,
///   "starterDecks": null,
///   "tickets": null,
///   "CustomTokens": {
///     "DraftToken": 19
///   },
///   "draftTokens": 19,
///   "sealedTokens": 0
/// }
/// ```
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlayerInventoryData {
    pub wc_common: u32,
    pub wc_uncommon: u32,
    pub wc_rare: u32,
    pub wc_mythic: u32,
    pub gold: u32,
    pub gems: u32,
    pub wc_track_position: i32,
    pub vault_progress: HashableF64,
    pub boosters: Vec<BoosterStack>,
    pub vouchers: Vec<ClientVoucherDescription>,
    pub basic_land_set: Option<String>,
    pub latest_basic_land_set: Option<String>,
    pub starter_decks: Option<Vec<Guid>>,
    pub tickets: Option<Vec<TicketStack>>,
    pub custom_tokens: Option<HashMap<String, u32>>,
}

impl Hash for PlayerInventoryData {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.wc_common.hash(state);
        self.wc_uncommon.hash(state);
        self.wc_rare.hash(state);
        self.wc_mythic.hash(state);
        self.gold.hash(state);
        self.gems.hash(state);
        self.wc_track_position.hash(state);
        self.vault_progress.hash(state);
        self.boosters.hash(state);
        self.vouchers.hash(state);
        self.basic_land_set.hash(state);
        self.latest_basic_land_set.hash(state);
        self.starter_decks.hash(state);
        self.tickets.hash(state);
        if self.custom_tokens.is_some() {
            for (key, value) in self.custom_tokens.as_ref().unwrap().iter() {
                key.hash(state);
                value.hash(state);
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClientVoucherDescription {
    pub image1: String,
    pub image2: String,
    pub image3: String,
    pub prefab: String,
    pub reference_id: String,
    pub header_loc_key: String,
    pub description_loc_key: String,
    pub quantity: String,
    pub loc_params: Option<HashMap<String, i32>>,
    pub available_date: String,
}

impl Hash for ClientVoucherDescription {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.image1.hash(state);
        self.image2.hash(state);
        self.image3.hash(state);
        self.prefab.hash(state);
        self.reference_id.hash(state);
        self.header_loc_key.hash(state);
        self.description_loc_key.hash(state);
        self.quantity.hash(state);
        if self.loc_params.is_some() {
            for (key, value) in self.loc_params.as_ref().unwrap().iter() {
                key.hash(state);
                value.hash(state);
            }
        }
        self.available_date.hash(state);
    }
}

/// `InventoryUpdateData` is emitted when an update happens to the player inventory eg. when a match ends.
/// It contains a delta of gems, xp, cards of the player inventory and it's source.
///
/// Example attachment in the event body:
/// ```
/// {
///   "delta": {
///     "gemsDelta": 0,
///     "goldDelta": 0,
///     "boosterDelta": [],
///     "cardsAdded": [],
///     "decksAdded": [],
///     "starterDecksAdded": null,
///     "vanityItemsAdded": [],
///     "vanityItemsRemoved": null,
///     "vaultProgressDelta": 0,
///     "wcTrackPosition": 0,
///     "wcCommonDelta": 0,
///     "wcUncommonDelta": 0,
///     "wcRareDelta": 0,
///     "wcMythicDelta": 0,
///     "artSkinsAdded": [
///       {
///         "artId": 426094,
///         "ccv": "DA"
///       }
///     ],
///     "artSkinsRemoved": null,
///     "voucherItemsDelta": [],
///     "tickets": null,
///     "customTokenDelta": [
///       {
///         "id": "BattlePass_DMU_Orb",
///         "delta": 1
///       }
///     ]
///   },
///   "aetherizedCards": [
///     {
///       "grpId": 82241,
///       "addedToInventory": true,
///       "isGrantedFromDeck": false,
///       "vaultProgress": 0,
///       "goldAwarded": 0,
///       "gemsAwarded": 0,
///       "set": "DMU"
///     }
///   ],
///   "xpGained": 0,
///   "context": {
///     "source": "CampaignGraphTieredRewardNode",
///     "sourceId": null
///   },
///   "parentcontext": null
/// }
/// ```
#[derive(Debug, Serialize, Deserialize, Hash, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InventoryUpdateData {
    delta: InventoryDelta,
    aetherized_cards: Vec<AetherizedCardInformation>,
    xp_gained: i32,
    context: InventoryUpdateContext,
    parent_context: Option<String>,
}

/// `InventoryDelta` describes the changes in the player inventory eg. when a match ends.
///
/// Example json in the update body:
///
/// ```
/// {
///   "gemsDelta": 0,
///   "goldDelta": 0,
///   "boosterDelta": [],
///   "cardsAdded": [],
///   "decksAdded": [],
///   "starterDecksAdded": null,
///   "vanityItemsAdded": [],
///   "vanityItemsRemoved": null,
///   "vaultProgressDelta": 0,
///   "wcTrackPosition": 0,
///   "wcCommonDelta": 0,
///   "wcUncommonDelta": 0,
///   "wcRareDelta": 0,
///   "wcMythicDelta": 0,
///   "artSkinsAdded": [
///     {
///       "artId": 426094,
///       "ccv": "DA"
///     }
///   ],
///   "artSkinsRemoved": null,
///   "voucherItemsDelta": [],
///   "tickets": null,
///   "customTokenDelta": [
///     {
///       "id": "BattlePass_DMU_Orb",
///       "delta": 1
///     }
///   ]
/// }
/// ```
#[derive(Debug, Serialize, Deserialize, Hash, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InventoryDelta {
    pub gems_delta: i32,
    pub gold_delta: i32,
    pub booster_delta: Vec<BoosterStack>,
    pub cards_added: Vec<u32>,
    pub decks_added: Vec<Guid>,
    pub starter_decks_added: Option<Vec<Guid>>,
    pub vanity_items_added: Vec<String>,
    pub vanity_items_removed: Option<Vec<String>>,
    pub vault_progress_delta: HashableF64,
    pub wc_track_position: i32,
    pub wc_common_delta: i32,
    pub wc_uncommon_delta: i32,
    pub wc_rare_delta: i32,
    pub wc_mythic_delta: i32,
    pub art_skins_added: Vec<ArtSkin>,
    pub art_skins_removed: Option<Vec<ArtSkin>>,
    pub tickets: Option<Vec<TicketStack>>,
    custom_token_delta: Vec<CustomTokenDeltaInfo>,
    voucher_items_delta: Option<Vec<VoucherStack>>,
}

/// The awarded card type in the inventory update event.
///
/// Example attachment in the json body:
/// ```
/// {
///   "grpId": 82241,
///   "addedToInventory": true,
///   "isGrantedFromDeck": false,
///   "vaultProgress": 0,
///   "goldAwarded": 0,
///   "gemsAwarded": 0,
///   "set": "DMU"
/// }
/// ```
#[derive(Debug, Serialize, Deserialize, Hash, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AetherizedCardInformation {
    pub grp_id: u32,
    pub added_to_inventory: bool,
    pub is_granted_from_deck: bool,
    pub vault_progress: HashableF64,
    pub gold_awarded: i32,
    pub gems_awarded: i32,
    // This is `None` for wildcards opened in boosters
    pub set: Option<String>,
}
impl AetherizedCardInformation {
    const COMMON_WILDCARD_GRPID: u32 = 9;
    const UNCOMMON_WILDCARD_GRPID: u32 = 8;
    const RARE_WILDCARD_GRPID: u32 = 7;
    const MYTHICRARE_WILDCARD_GRPID: u32 = 6;

    fn is_actual_card(&self) -> bool {
        return self.grp_id != AetherizedCardInformation::COMMON_WILDCARD_GRPID
            && self.grp_id != AetherizedCardInformation::UNCOMMON_WILDCARD_GRPID
            && self.grp_id != AetherizedCardInformation::RARE_WILDCARD_GRPID
            && self.grp_id != AetherizedCardInformation::MYTHICRARE_WILDCARD_GRPID;
    }
}

/// The context object for an inventory update.
///
/// It has a source field which is an enum in the MTGA client model, describing what triggered the inventory change,
/// eg. it was a reward for a mastery pass or some cards were banned etc...
///
/// Example attachment in the json body:
/// ```
/// {
///   "source": "CampaignGraphTieredRewardNode",
///   "sourceId": null
/// }
/// ```
///
/// The current enum values of `Wizards.Models.InventoryUpdateSource` (as of 2023-10-19) are:
/// `Unknown`, `QuestReward`, `DailyWins`, `WeeklyWins`, `LoginGrant`, `BattlePassLevelUp`, `BattlePassLevelMasteryTree`,
/// `EarlyPlayerProgressionLevelUp`, `EarlyPlayerProgressionMasteryTree`, `ProgressionRewardTierAdd`, `RenewalReward`,
/// `MercantilePurchase`, `MercantileChestPurchase`, `MercantileBoosterPurchase`, `EventReward`, `RedeemVoucher`,
/// `ModifyPlayerInventory`, `OpenChest`, `MassOpenChest`, `BasicLandSetUpdate`, `CompleteVault`, `CosmeticPurchase`,
/// `WildCardRedemption`, `BoosterOpen`, `StarterDeckUpgrade`, `RankedSeasonReward`, `EventPayEntry`, `BannedCardGrant`,
/// `EventEntryReward`, `CatalogPurchase`, `CampaignGraphReward`, `EventRefundEntry`, `Cleanup`, `IdEmpotentLoginGrant`,
/// `CustomerSupportGrant`, `EntryReward`, `EventGrantCardPool`, `CampaignGraphPayoutNode`,
/// `CampaignGraphAutomaticPayoutNode`, `CampaignGraphPurchaseNode`, `CampaignGraphTieredRewardNode`,
/// `AccumulativePayoutNode`, `Letter`.
#[derive(Debug, Serialize, Deserialize, Hash, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InventoryUpdateContext {
    pub source: String,
    pub source_id: Option<String>,
}

/// Card skin type in the inventory update event.
///
/// Example attachment in the json body:
/// ```
/// {
///   "artId": 426094,
///   "ccv": "DA"
/// }
/// ```
#[derive(Debug, Serialize, Deserialize, Hash, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ArtSkin {
    pub art_id: u32,
    pub ccv: String,
}

impl ArtSkin {
    pub fn logview_name(&self) -> String {
        format!("{} - {}", self.ccv, self.art_id)
    }
}

/// `TicketStack` describes a new ticket added or removed in an inventory update event.
#[derive(Debug, Serialize, Deserialize, Hash, Clone)]
pub struct TicketStack {
    pub ticket: String,
    pub count: i32,
}

/// `VoucherStack` describes a new voucher added or removed in an inventory update event.
#[derive(Debug, Serialize, Deserialize, Hash, Clone)]
pub struct VoucherStack {
    #[serde(rename = "Id")]
    pub id: Guid,
    pub count: i32,
}

/// `CustomTokenDeltaInfo` describes the special tokens in the MTGA client when an inventory update event occurs.
/// These can be like the mastery orbs etc...
///
/// Example attachment in the json body:
/// ```
/// {
///   "id": "BattlePass_DMU_Orb",
///   "delta": 1
/// }
/// ```
#[derive(Debug, Serialize, Deserialize, Hash, Clone)]
pub struct CustomTokenDeltaInfo {
    pub id: String,
    pub delta: i32,
}

impl CustomTokenDeltaInfo {
    pub fn is_draft_token(&self) -> bool {
        self.id == "DraftToken"
    }

    pub fn is_orb_token(&self) -> bool {
        self.id.ends_with("_Orb")
    }
}

/// `BoosterStack` describes the pack of cards in the inventory update event.
///
/// Example attachment in the json body:
/// ```
/// {
///   "collationId": 100023,
///   "count": 1
/// }
/// ```
#[derive(Debug, Serialize, Deserialize, Hash, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BoosterStack {
    pub collation_id: i32,
    pub count: i32,
}

impl BoosterStack {
    pub fn short_name(&self) -> String {
        // This is coming from the `Wotc.Mtga.Wrapper.CollationMapping` type
        match self.collation_id {
            62242 => String::from("kld"),
            62979 => String::from("aer"),
            100003 => String::from("akh"),
            100004 => String::from("hou"),
            100005 => String::from("xln"),
            100006 => String::from("rix"),
            100007 => String::from("dar"),
            100008 => String::from("m19"),
            100009 => String::from("grn"),
            100010 => String::from("rna"),
            100013 => String::from("war"),
            100014 => String::from("m20"),
            100015 => String::from("eld"),
            100016 => String::from("thb"),
            100017 => String::from("iko"),
            100018 => String::from("m21"),
            100019 => String::from("akr"),
            100020 => String::from("znr"),
            100021 => String::from("klr"),
            100022 => String::from("khm"),
            100023 => String::from("stx"),
            100024 => String::from("afr"),
            100025 => String::from("mid"),
            100026 => String::from("vow"),
            100027 => String::from("neo"),
            100028 => String::from("snc"),
            100029 => String::from("hbg"),
            100030 => String::from("dmu"),
            100031 => String::from("bro"),
            100032 => String::from("one"),
            100033 => String::from("sir"),
            100037 => String::from("mom"),
            100038 => String::from("mat"),
            100039 => String::from("ltr"),
            100040 => String::from("woe"),
            100041 => String::from("lci"),
            100042 => String::from("ktk"),
            100043 => String::from("mkm"),
            100044 => String::from("otj"),
            200001 => String::from("kld_draft"),
            200002 => String::from("aer_draft"),
            200003 => String::from("akh_draft"),
            200004 => String::from("hou_draft"),
            200005 => String::from("xln_draft"),
            200006 => String::from("rix_draft"),
            200007 => String::from("dar_draft"),
            200008 => String::from("m19_draft"),
            200009 => String::from("grn_draft"),
            200010 => String::from("rna_draft"),
            200013 => String::from("war_draft"),
            200014 => String::from("m20_draft"),
            200015 => String::from("eld_draft"),
            200016 => String::from("thb_draft"),
            200017 => String::from("iko_draft"),
            200018 => String::from("m21_draft"),
            200019 => String::from("akr_draft"),
            200020 => String::from("znr_draft"),
            200021 => String::from("klr_draft"),
            200022 => String::from("khm_draft"),
            200023 => String::from("stx_draft"),
            200024 => String::from("afr_draft"),
            200025 => String::from("mid_draft"),
            200026 => String::from("vow_draft"),
            200027 => String::from("neo_draft"),
            200028 => String::from("snc_draft"),
            200029 => String::from("hbg_draft"),
            200030 => String::from("dmu_draft"),
            200031 => String::from("bro_draft"),
            200032 => String::from("one_draft"),
            200033 => String::from("sir_bonus_typal"),
            200034 => String::from("sir_bonus_flashback"),
            200035 => String::from("sir_bonus_morbid"),
            200036 => String::from("sir_bonus_allstar"),
            200037 => String::from("mom_draft"),
            200038 => String::from("mat_draft"),
            200039 => String::from("ltr_draft"),
            200040 => String::from("woe_draft"),
            200041 => String::from("lci_draft"),
            200042 => String::from("ktk_draft"),
            200043 => String::from("mkm_draft"),
            200044 => String::from("otj_draft"),
            300000 => String::from("cube"),
            300001 => String::from("tinkererscube"),
            300002 => String::from("chromaticcube"),
            300003 => String::from("dbl"),
            300004 => String::from("mixedup_1"),
            300005 => String::from("mixedup_2"),
            300006 => String::from("mixedup_3"),
            300007 => String::from("creaturecube"),
            300010 => String::from("test_draft_1"),
            300011 => String::from("test_draft_2"),
            300012 => String::from("test_draft_3"),
            300020 => String::from("remixdraft_artifact"),
            400000 => String::from("alchemy"),
            400026 => String::from("y22_mid"),
            400027 => String::from("y22_neo"),
            400028 => String::from("y22_snc"),
            400030 => String::from("y23_dmu"),
            400031 => String::from("y23_bro"),
            400032 => String::from("y23_one"),
            400040 => String::from("y24_woe"),
            400041 => String::from("y24_lci"),
            400043 => String::from("y24_mkm"),
            500000 => String::from("mythic"),
            500015 => String::from("eld_mythic"),
            500016 => String::from("thb_mythic"),
            500017 => String::from("iko_mythic"),
            500018 => String::from("m21_mythic"),
            500020 => String::from("znr_mythic"),
            500022 => String::from("khm_mythic"),
            500023 => String::from("stx_mythic"),
            500024 => String::from("afr_mythic"),
            500025 => String::from("mid_mythic"),
            500026 => String::from("vow_mythic"),
            500027 => String::from("neo_mythic"),
            500028 => String::from("snc_mythic"),
            500029 => String::from("hbg_mythic"),
            500030 => String::from("dmu_mythic"),
            500031 => String::from("bro_mythic"),
            500032 => String::from("one_mythic"),
            500033 => String::from("sir_mythic"),
            500037 => String::from("mom_mythic"),
            500039 => String::from("ltr_mythic"),
            500040 => String::from("woe_mythic"),
            500041 => String::from("lci_mythic"),
            500042 => String::from("ktk_mythic"),
            500043 => String::from("mkm_mythic"),
            500044 => String::from("otj_mythic"),
            600028 => String::from("y22_snc_draft"),
            600030 => String::from("y23_dmu_draft"),
            600031 => String::from("y23_bro_draft"),
            600032 => String::from("y23_one_draft"),
            600040 => String::from("y24_woe_draft"),
            600041 => String::from("y24_lci_draft"),
            600043 => String::from("y24_mkm_draft"),
            700028 => String::from("snc_rebalanced"),
            900980 => String::from("goldenbooster_standard"),
            999999 => String::from("futuresetplaceholder"),
            _ => String::from("unknown"),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Hash, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CollectedCard {
    pub grp_id: u32,
    pub count: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MtgaCard {
    name: String,

    #[serde(default)]
    scry_name: String,
    set: String,

    #[serde(default)]
    scry_set: String,

    arena_id: u32,
    is_primary_card: bool,
    is_main_set: bool,
    is_token: bool,
    linked_faces: Vec<u32>,
    is_collectible: bool,
    is_craftable: bool,
    tokens: Vec<u32>,
    templates: Vec<u32>,
    is_rebalanced: bool,
    rebalanced_card_link: u32,
    artist: String,
    art_id: u32,
    collector_number: String,
    linked_face_type: String,
    max_collected: u32,

    #[serde(default)]
    scry_uri: Uri,
}

impl MtgaCard {
    pub fn arena_id(&self) -> u32 {
        self.arena_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn lowercase_artist(&self) -> String {
        self.artist.to_lowercase()
    }

    pub fn set(&self) -> &str {
        &self.set
    }

    pub fn linked_faces(&self) -> &[u32] {
        &self.linked_faces
    }

    pub fn linked_face_type(&self) -> &str {
        &self.linked_face_type
    }

    pub fn collector_number(&self) -> &str {
        &self.collector_number
    }

    pub fn is_collectible(&self) -> bool {
        self.is_collectible
    }

    pub fn max_collected(&self) -> u32 {
        self.max_collected
    }
}

#[derive(Debug)]
pub struct TrackerCard {
    name: String,
    set: String,
    collector_number: String,
    scry_uri: Uri,
    arena_id: u32,
    image_uri: Uri,
    rarity: String,
    in_booster: bool,
    is_alchemy_card: bool,
    max_collected: u32,
}

impl TrackerCard {
    pub fn new(mtga_card: &MtgaCard, scry_card: &ScryCard) -> TrackerCard {
        TrackerCard {
            name: String::from(scry_card.name()),
            set: String::from(scry_card.set()),
            collector_number: String::from(mtga_card.collector_number()),
            scry_uri: scry_card.scryfall_uri().to_owned(),
            arena_id: mtga_card.arena_id(),
            image_uri: scry_card.get_image_uri().to_owned(),
            rarity: String::from(scry_card.rarity()),
            in_booster: scry_card.booster(),
            is_alchemy_card: (mtga_card.rebalanced_card_link != 0 && mtga_card.is_rebalanced),
            max_collected: mtga_card.max_collected(),
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn set(&self) -> &str {
        self.set.as_ref()
    }

    pub fn scry_uri(&self) -> &Uri {
        &self.scry_uri
    }

    pub fn arena_id(&self) -> u32 {
        self.arena_id
    }

    pub fn image_uri(&self) -> &Uri {
        &self.image_uri
    }

    pub fn rarity(&self) -> &str {
        self.rarity.as_ref()
    }

    pub fn in_booster(&self) -> bool {
        self.in_booster
    }

    pub fn collector_number(&self) -> &str {
        self.collector_number.as_ref()
    }

    pub fn max_collected(&self) -> u32 {
        self.max_collected
    }

    pub fn get_by_id(db: &Connection, arena_id: u32) -> Result<TrackerCard> {
        let mut stmt = db.prepare(
            "SELECT cards_db.'name',
                        cards_db.'set',
                        cards_db.'collector_number',
                        cards_db.'scry_uri',
                        cards_db.'arena_id',
                        cards_db.'image_uri',
                        cards_db.'rarity',
                        cards_db.'in_booster',
                        cards_db.'is_alchemy_card',
                        cards_db.'max_collected'
                FROM cards_db
                WHERE cards_db.'arena_id' = ?1",
        )?;

        let mut results = stmt.query(rusqlite::params![arena_id])?;
        if let Some(row) = results.next()? {
            let card = TrackerCard {
                name: row.get(0)?,
                set: row.get(1)?,
                collector_number: row.get(2)?,
                scry_uri: row.get(3)?,
                arena_id,
                image_uri: row.get(5)?,
                rarity: row.get(6)?,
                in_booster: row.get(7)?,
                is_alchemy_card: row.get(8)?,
                max_collected: row.get(9)?,
            };
            Ok(card)
        } else {
            Err("Invalid arena_id".into())
        }
    }

    pub fn get_all_cards(db: &Connection) -> Result<HashMap<u32, TrackerCard>> {
        let mut tracker_cards = HashMap::new();

        let mut stmt = db.prepare(
            "SELECT cards_db.'name',
                        cards_db.'set',
                        cards_db.'collector_number',
                        cards_db.'scry_uri',
                        cards_db.'arena_id',
                        cards_db.'image_uri',
                        cards_db.'rarity',
                        cards_db.'in_booster',
                        cards_db.'is_alchemy_card',
                        cards_db.'max_collected'
                FROM cards_db",
        )?;

        let rows = stmt.query_map([], |row| {
            let name = row.get(0)?;
            let set = row.get(1)?;
            let collector_number = row.get(2)?;
            let scry_uri = row.get(3)?;
            let arena_id = row.get(4)?;
            let image_uri = row.get(5)?;
            let rarity = row.get(6)?;
            let in_booster = row.get(7)?;
            let is_alchemy_card = row.get(8)?;
            let max_collected = row.get(9)?;

            Ok(TrackerCard {
                name,
                set,
                collector_number,
                scry_uri,
                arena_id,
                image_uri,
                rarity,
                in_booster,
                is_alchemy_card,
                max_collected,
            })
        })?;

        for row in rows {
            let tracker_card = row?;
            tracker_cards.insert(tracker_card.arena_id(), tracker_card);
        }

        Ok(tracker_cards)
    }

    pub fn get_all_cards_from_set(db: &Connection, set: &str) -> Result<HashMap<u32, TrackerCard>> {
        let mut tracker_cards = HashMap::new();

        let mut stmt = db.prepare(
            "SELECT cards_db.'name',
                        cards_db.'set',
                        cards_db.'collector_number',
                        cards_db.'scry_uri',
                        cards_db.'arena_id',
                        cards_db.'image_uri',
                        cards_db.'rarity',
                        cards_db.'in_booster',
                        cards_db.'is_alchemy_card',
                        cards_db.'max_collected'
                FROM cards_db
                WHERE cards_db.'set' = ?1",
        )?;

        let rows = stmt.query_map([set], |row| {
            let name = row.get(0)?;
            let set = row.get(1)?;
            let collector_number = row.get(2)?;
            let scry_uri = row.get(3)?;
            let arena_id = row.get(4)?;
            let image_uri = row.get(5)?;
            let rarity = row.get(6)?;
            let in_booster = row.get(7)?;
            let is_alchemy_card = row.get(8)?;
            let max_collected = row.get(9)?;

            Ok(TrackerCard {
                name,
                set,
                collector_number,
                scry_uri,
                arena_id,
                image_uri,
                rarity,
                in_booster,
                is_alchemy_card,
                max_collected,
            })
        })?;

        for row in rows {
            let tracker_card = row?;
            tracker_cards.insert(tracker_card.arena_id(), tracker_card);
        }

        Ok(tracker_cards)
    }

    pub fn is_alchemy_card(&self) -> bool {
        self.is_alchemy_card
    }
}

impl Display for TrackerCard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} - ({}:#{}) (#{}) ({})",
            self.name, self.set, self.collector_number, self.arena_id, self.rarity
        )
    }
}

#[derive(Debug, Clone)]
pub struct HashableF64 {
    inner_value: f64,
}

impl Serialize for HashableF64 {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_f64(self.inner_value)
    }
}

impl<'de> Deserialize<'de> for HashableF64 {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_f64(F64Visitor)
    }
}

struct F64Visitor;

impl<'de> Visitor<'de> for F64Visitor {
    type Value = HashableF64;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a float value")
    }

    fn visit_f64<E>(self, v: f64) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(HashableF64 { inner_value: v })
    }
}

impl Hash for HashableF64 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        integer_decode(self.inner_value).hash(state)
    }
}

fn integer_decode(val: f64) -> (u64, i16, i8) {
    let bits: u64 = val.to_bits();
    let sign: i8 = if bits >> 63 == 0 { 1 } else { -1 };
    let mut exponent: i16 = ((bits >> 52) & 0x7ff) as i16;
    let mantissa = if exponent == 0 {
        (bits & 0xfffffffffffff) << 1
    } else {
        (bits & 0xfffffffffffff) | 0x10000000000000
    };

    exponent -= 1023 + 52;
    (mantissa, exponent, sign)
}
