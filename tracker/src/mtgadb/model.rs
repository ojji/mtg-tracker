//! The main model types are `InventoryUpdateEvent` and `CollectionEvent`, appearing as the main JSON objects in the
//! log file. Every main model type has a `Timestamp` and an `Attachment` field, describing when the event occured and
//! containing the detailed event object.

use std::{collections::HashMap, error::Error, fmt::Display};

use rusqlite::{types::FromSql, Connection, ToSql};
use serde::{Deserialize, Serialize};

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

    ///	The Scryfall ID for the card back design present on this card.
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

    ///	A link to this card’s set on Scryfall’s website.
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

    pub fn lowercase_name(&self) -> String {
        self.name.to_lowercase()
    }

    pub fn is_available_in_arena(&self) -> bool {
        self.games.contains(&String::from("arena"))
    }

    pub fn lowercase_artist(&self) -> Option<String> {
        match &self.artist {
            Some(artist) => Some(artist.to_lowercase()),
            None => None,
        }
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
            .get("png")
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

#[derive(Clone, Debug, Deserialize, Serialize)]
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

/*

[MTGADataCollector][inventory]
{
  "Timestamp": "2022-09-11T16:28:55.8865732+02:00",
  "Attachment": {
    "wcCommon": 38,
    "wcUncommon": 71,
    "wcRare": 45,
    "wcMythic": 30,
    "gold": 793775,
    "gems": 25080,
    "wcTrackPosition": 0,
    "vaultProgress": 27.1,
    "boosters": [
      {
        "collationId": 100023,
        "count": 65
      },
      {
        "collationId": 100024,
        "count": 51
      },
      {
        "collationId": 100025,
        "count": 50
      },
      {
        "collationId": 100026,
        "count": 58
      },
      {
        "collationId": 400026,
        "count": 9
      },
      {
        "collationId": 100027,
        "count": 53
      },
      {
        "collationId": 100028,
        "count": 40
      },
      {
        "collationId": 400028,
        "count": 6
      },
      {
        "collationId": 100029,
        "count": 37
      },
      {
        "collationId": 100030,
        "count": 15
      }
    ],
    "vouchers": [],
    "basicLandSet": null,
    "latestBasicLandSet": null,
    "starterDecks": null,
    "tickets": null,
    "CustomTokens": {
      "DraftToken": 19
    },
    "draftTokens": 19,
    "sealedTokens": 0
  }
}
*/

/// `InventoryUpdateEvent` is the outermost json object describing an inventory update event parsed from the log file.
///
/// Example object in the json body:
/// ```
/// {
///   "Timestamp": "2022-09-11T17:00:35.9879457+02:00",
///   "Attachment": {
///     "delta": {
///       "gemsDelta": 0,
///       "goldDelta": 100,
///       "boosterDelta": [],
///       "cardsAdded": [],
///       "decksAdded": [],
///       "starterDecksAdded": null,
///       "vanityItemsAdded": [],
///       "vanityItemsRemoved": null,
///       "vaultProgressDelta": 0,
///       "wcTrackPosition": 0,
///       "wcCommonDelta": 0,
///       "wcUncommonDelta": 0,
///       "wcRareDelta": 0,
///       "wcMythicDelta": 0,
///       "artSkinsAdded": [],
///       "artSkinsRemoved": null,
///       "voucherItemsDelta": [],
///       "tickets": null,
///       "customTokenDelta": []
///     },
///     "aetherizedCards": [],
///     "xpGained": 25,
///     "context": {
///       "source": "DailyWins",
///       "sourceId": null
///     },
///     "parentcontext": null
///   }
/// }
/// ```
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct InventoryUpdateEvent {
    pub timestamp: String,
    pub attachment: ClientInventoryUpdateReportItem,
}

/// `ClientInventoryUpdateReportItem` is emitted when an update happens to the player inventory eg. when a match ends.
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
#[derive(Debug, Serialize, Deserialize)]
pub struct ClientInventoryUpdateReportItem {
    delta: InventoryDelta,

    #[serde(rename = "aetherizedCards")]
    aetherized_cards: Vec<AetherizedCardInformation>,

    #[serde(rename = "xpGained")]
    xp_gained: i32,

    context: InventoryUpdateContext,

    #[serde(rename = "parentcontext")]
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
#[derive(Debug, Serialize, Deserialize)]
pub struct InventoryDelta {
    #[serde(rename = "gemsDelta")]
    pub gems_delta: i32,

    #[serde(rename = "goldDelta")]
    pub gold_delta: i32,

    #[serde(rename = "boosterDelta")]
    pub booster_delta: Vec<BoosterStack>,

    #[serde(rename = "cardsAdded")]
    pub cards_added: Vec<u32>,

    #[serde(rename = "decksAdded")]
    pub decks_added: Vec<Guid>,

    #[serde(rename = "starterDecksAdded")]
    pub starter_decks_added: Option<Vec<Guid>>,

    #[serde(rename = "vanityItemsAdded")]
    pub vanity_items_added: Vec<String>,

    #[serde(rename = "vanityItemsRemoved")]
    pub vanity_items_removed: Option<Vec<String>>,

    #[serde(rename = "vaultProgressDelta")]
    pub vault_progress_delta: f32,

    #[serde(rename = "wcTrackPosition")]
    pub wc_track_position: i32,

    #[serde(rename = "wcCommonDelta")]
    pub wc_common_delta: i32,

    #[serde(rename = "wcUncommonDelta")]
    pub wc_uncommon_delta: i32,

    #[serde(rename = "wcRareDelta")]
    pub wc_rare_delta: i32,

    #[serde(rename = "wcMythicDelta")]
    pub wc_mythic_delta: i32,

    #[serde(rename = "artSkinsAdded")]
    pub art_skins_added: Vec<ArtSkin>,

    #[serde(rename = "artSkinsRemoved")]
    pub art_skins_removed: Option<Vec<ArtSkin>>,

    pub tickets: Option<Vec<TicketStack>>,

    #[serde(rename = "customTokenDelta")]
    custom_token_delta: Vec<CustomTokenDeltaInfo>,

    #[serde(rename = "voucherItemsDelta")]
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
#[derive(Debug, Serialize, Deserialize)]
pub struct AetherizedCardInformation {
    #[serde(rename = "grpId")]
    pub grp_id: u32,

    #[serde(rename = "addedToInventory")]
    pub added_to_inventory: bool,

    #[serde(rename = "isGrantedFromDeck")]
    pub is_granted_from_deck: bool,

    #[serde(rename = "vaultProgress")]
    pub vault_progress: f32,

    #[serde(rename = "goldAwarded")]
    pub gold_awarded: i32,

    #[serde(rename = "gemsAwarded")]
    pub gems_awarded: i32,

    pub set: String,
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
/// The current enum values (as of 2022-09-12) are:
/// `Unknown`, `QuestReward`, `DailyWins`, `WeeklyWins`, `LoginGrant`, `BattlePassLevelUp`, `BattlePassLevelMasteryTree`,
/// `EarlyPlayerProgressionLevelUp`, `EarlyPlayerProgressionMasteryTree`, `ProgressionRewardTierAdd`, `RenewalReward`,
/// `MercantilePurchase`, `MercantileChestPurchase`, `MercantileBoosterPurchase`, `EventReward`, `RedeemVoucher`,
/// `ModifyPlayerInventory`, `OpenChest`, `MassOpenChest`, `BasicLandSetUpdate`, `CompleteVault`, `CosmeticPurchase`,
/// `WildCardRedemption`, `BoosterOpen`, `StarterDeckUpgrade`, `RankedSeasonReward`, `EventPayEntry`, `BannedCardGrant`,
/// `EventEntryReward`, `CatalogPurchase`, `CampaignGraphReward`, `EventRefundEntry`, `Cleanup`, `IdEmpotentLoginGrant`,
/// `CustomerSupportGrant`, `EntryReward`, `EventGrantCardPool`, `CampaignGraphPayoutNode`,
/// `CampaignGraphAutomaticPayoutNode`, `CampaignGraphPurchaseNode`, `CampaignGraphTieredRewardNode`,
/// `AccumulativePayoutNode`, `Letter`.
#[derive(Debug, Serialize, Deserialize)]
pub struct InventoryUpdateContext {
    pub source: String,

    #[serde(rename = "sourceId")]
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
#[derive(Debug, Serialize, Deserialize)]
pub struct ArtSkin {
    #[serde(rename = "artId")]
    pub art_id: u32,

    pub ccv: String,
}

/// `TicketStack` describes a new ticket added or removed in an inventory update event.
#[derive(Debug, Serialize, Deserialize)]
pub struct TicketStack {
    pub ticket: String,
    pub count: i32,
}

/// `VoucherStack` describes a new voucher added or removed in an inventory update event.
#[derive(Debug, Serialize, Deserialize)]
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
#[derive(Debug, Serialize, Deserialize)]
pub struct CustomTokenDeltaInfo {
    pub id: String,
    pub delta: i32,
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
#[derive(Debug, Serialize, Deserialize)]
pub struct BoosterStack {
    #[serde(rename = "collationId")]
    pub collation_id: i32,
    pub count: i32,
}

impl BoosterStack {
    pub fn name_from_collation_id(&self) -> String {
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
            300000 => String::from("cube"),
            300001 => String::from("tinkererscube"),
            300002 => String::from("chromaticcube"),
            300003 => String::from("dbl"),
            300004 => String::from("mixedup_1"),
            300005 => String::from("mixedup_2"),
            300006 => String::from("mixedup_3"),
            400026 => String::from("y22_mid"),
            400027 => String::from("y22_neo"),
            400028 => String::from("y22_snc"),
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
            _ => String::from("unknown"),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CollectionEvent {
    pub timestamp: String,
    pub attachment: Vec<CollectedCard>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectedCard {
    pub grp_id: u32,
    pub count: i32,
    pub is_rebalanced: bool,
    pub rebalanced_card_link: u32,
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

    pub fn get_all(db: &Connection) -> Result<HashMap<u32, TrackerCard>, Box<dyn Error>> {
        let mut tracker_cards = HashMap::new();

        let mut stmt = db.prepare(
            "SELECT cards_db.'name',
                        cards_db.'set',
                        cards_db.'collector_number',
                        cards_db.'scry_uri',
                        cards_db.'arena_id',
                        cards_db.'image_uri',
                        cards_db.'rarity',
                        cards_db.'in_booster'
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

            Ok(TrackerCard {
                name,
                set,
                collector_number,
                scry_uri,
                arena_id,
                image_uri,
                rarity,
                in_booster,
            })
        })?;

        for row in rows {
            let tracker_card = row?;
            tracker_cards.insert(tracker_card.arena_id(), tracker_card);
        }

        Ok(tracker_cards)
    }
}
