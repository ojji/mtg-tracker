use std::collections::HashMap;

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

#[derive(Debug, Deserialize, Serialize)]
pub struct ScryCard {
    /// This card’s Arena ID, if any. A large percentage of cards are not available on Arena and do not have this ID.
    pub arena_id: Option<u32>,

    /// A unique ID for this card in Scryfall’s database.
    pub id: Guid,

    /// A [language](https://scryfall.com/docs/api/languages) code for this printing.
    pub lang: String,

    /// This card’s Magic Online ID (also known as the Catalog ID), if any.
    /// A large percentage of cards are not available on Magic Online and do not have this ID.
    pub mtgo_id: Option<u32>,

    /// This card’s foil Magic Online ID (also known as the Catalog ID), if any.
    /// A large percentage of cards are not available on Magic Online and do not have this ID.
    pub mtgo_foil_id: Option<u32>,

    /// This card’s multiverse IDs on Gatherer, if any, as an array of integers. Note that Scryfall includes many
    /// promo cards, tokens, and other esoteric objects that do not have these identifiers.
    pub multiverse_ids: Option<Vec<u32>>,

    /// This card’s ID on [TCGplayer’s API](https://docs.tcgplayer.com/docs), also known as the `productId`.
    pub tcgplayer_id: Option<u32>,

    /// This card’s ID on [TCGplayer’s API](https://docs.tcgplayer.com/docs), for its etched version if that version
    /// is a separate product.
    pub tcgplayer_etched_id: Option<u32>,

    /// This card’s ID on Cardmarket’s API, also known as the `idProduct`.
    pub cardmarket_id: Option<u32>,

    /// A content type for this object, always `card`.
    pub object: String,

    /// A unique ID for this card’s oracle identity. This value is consistent across reprinted card editions, and unique
    /// among different cards with the same name (tokens, Unstable variants, etc).
    pub oracle_id: Option<Guid>,

    /// A link to where you can begin paginating all re/prints for this card on Scryfall’s API.
    pub prints_search_uri: Uri,

    /// A link to this card’s [rulings list](https://scryfall.com/docs/api/rulings) on Scryfall’s API.
    pub rulings_uri: Uri,

    /// A link to this card’s permapage on Scryfall’s website.
    pub scryfall_uri: Uri,

    /// A link to this card object on Scryfall’s API.
    pub uri: Uri,

    /// If this card is closely related to other cards, this property will be an array with Related Card Objects.
    pub all_parts: Option<Vec<ScryRelatedCard>>,

    /// An array of Card Face objects, if this card is multifaced.
    pub card_faces: Option<Vec<ScryCardFace>>,

    /// The card’s converted mana cost. Note that some funny cards have fractional mana costs.
    pub cmc: Option<f32>,

    /// This card’s color identity.
    pub color_identity: Colors,

    /// The colors in this card’s color indicator, if any. A null value for this field indicates the card does not have
    /// one.
    pub color_indicator: Option<Colors>,

    /// This card’s colors, if the overall card has colors defined by the rules. Otherwise the colors will be on
    /// the card_faces objects, see below.
    pub colors: Option<Colors>,

    /// This card’s overall rank/popularity on EDHREC. Not all cards are ranked.
    pub edhrec_rank: Option<u32>,

    /// This card’s hand modifier, if it is Vanguard card. This value will contain a delta, such as `-1`.
    pub hand_modifier: Option<String>,

    /// An array of keywords that this card uses, such as `'Flying'` and `'Cumulative upkeep'`.
    pub keywords: Vec<String>,

    /// A code for this card’s [layout](https://scryfall.com/docs/api/layouts).
    pub layout: String,

    /// An object describing the legality of this card across play formats. Possible legalities are `legal`,
    /// `not_legal`, `restricted`, and `banned`.
    pub legalities: HashMap<String, Legality>,

    /// This card’s life modifier, if it is Vanguard card. This value will contain a delta, such as `+2`.
    pub life_modifier: Option<String>,

    /// This loyalty if any. Note that some cards have loyalties that are not numeric, such as `X`.
    pub loyalty: Option<String>,

    /// The mana cost for this card. This value will be any empty string `""` if the cost is absent. Remember that per
    /// the game rules, a missing mana cost and a mana cost of `{0}` are different values.
    /// Multi-faced cards will report this value in card faces.
    pub mana_cost: Option<String>,

    /// The name of this card. If this card has multiple faces, this field will contain both names separated by `␣//␣`.
    pub name: String,

    /// The Oracle text for this card, if any.
    pub oracle_text: Option<String>,

    /// True if this card is oversized.
    pub oversized: bool,

    /// This card’s rank/popularity on Penny Dreadful. Not all cards are ranked.
    pub penny_rank: Option<u32>,

    /// This card’s power, if any. Note that some cards have powers that are not numeric, such as `*`.
    pub power: Option<String>,

    /// Colors of mana that this card could produce.
    pub produced_mana: Option<Colors>,

    /// True if this card is on the Reserved List.
    pub reserved: bool,

    /// This card’s toughness, if any. Note that some cards have toughnesses that are not numeric, such as `*`.
    pub toughness: Option<String>,

    /// The type line of this card.
    pub type_line: Option<String>,

    /// The name of the illustrator of this card. Newly spoiled cards may not have this field yet.
    pub artist: Option<String>,

    /// Whether this card is found in boosters.
    pub booster: bool,

    /// This card’s border color: `black`, `white`, `borderless`, `silver`, or `gold`.
    pub border_color: String,

    ///	The Scryfall ID for the card back design present on this card.
    pub card_back_id: Option<Guid>,

    /// This card’s collector number. Note that collector numbers can contain non-numeric characters,
    /// such as letters or `★`.
    pub collector_number: String,

    /// True if you should consider [avoiding use of this print](https://scryfall.com/blog/regarding-wotc-s-recent-statement-on-depictions-of-racism-220) downstream.
    pub content_warning: Option<bool>,

    /// True if this card was only released in a video game.
    pub digital: bool,

    /// An array of computer-readable flags that indicate if this card can come in `foil`, `nonfoil`, `etched`,
    /// or `glossy` finishes.
    pub finishes: Vec<String>,

    /// The just-for-fun name printed on the card (such as for Godzilla series cards).
    pub flavor_name: Option<String>,

    /// The flavor text, if any.
    pub flavor_text: Option<String>,

    /// This card’s [frame effects](https://scryfall.com/docs/api/layouts), if any.
    pub frame_effects: Option<Vec<String>>,

    /// This card’s [frame layout](https://scryfall.com/docs/api/layouts).
    pub frame: String,

    /// True if this card’s artwork is larger than normal.
    pub full_art: bool,

    /// A list of games that this card print is available in, `paper`, `arena`, and/or `mtgo`.
    pub games: Vec<String>,

    /// True if this card’s imagery is high resolution.
    pub highres_image: bool,

    /// A unique identifier for the card artwork that remains consistent across reprints. Newly spoiled cards may not have this field yet.
    pub illustration_id: Option<Guid>,

    /// A computer-readable indicator for the state of this card’s image, one of `missing`, `placeholder`, `lowres`,
    /// or `highres_scan`.
    pub image_status: String,

    /// An object listing available imagery for this card. See the [Card Imagery](https://scryfall.com/docs/api/images) article for more information.
    pub image_uris: Option<HashMap<String, Uri>>,

    /// An object containing daily price information for this card, including `usd`, `usd_foil`, `usd_etched`, `eur`,
    /// and `tix` prices, as strings.
    pub prices: HashMap<String, Option<String>>,

    /// The localized name printed on this card, if any.
    pub printed_name: Option<String>,

    /// The localized text printed on this card, if any.
    pub printed_text: Option<String>,

    /// The localized type line printed on this card, if any.
    pub printed_type_line: Option<String>,

    /// True if this card is a promotional print.
    pub promo: bool,

    /// An array of strings describing what categories of promo cards this card falls into.
    pub promo_types: Option<Vec<String>>,

    // purchase_uris: Object 	/// An object providing URIs to this card’s listing on major marketplaces. (seems missing)
    /// This card’s rarity. One of `common`, `uncommon`, `rare`, `special`, `mythic`, or `bonus`.
    pub rarity: String,

    /// An object providing URIs to this card’s listing on other Magic: The Gathering online resources.
    pub related_uris: HashMap<String, Uri>,

    /// The date this card was first released.
    pub released_at: Date,

    /// True if this card is a reprint.
    pub reprint: bool,

    ///	A link to this card’s set on Scryfall’s website.
    pub scryfall_set_uri: Uri,

    /// This card’s full set name.
    pub set_name: String,

    /// A link to where you can begin paginating this card’s set on the Scryfall API.
    pub set_search_uri: Uri,

    /// The type of set this printing is in.
    pub set_type: String,

    /// A link to this card’s [set object](https://scryfall.com/docs/api/sets) on Scryfall’s API.
    pub set_uri: Uri,

    /// This card’s set code.
    pub set: String,

    /// This card’s Set object UUID.
    pub set_id: Guid,

    /// True if this card is a Story Spotlight.
    pub story_spotlight: bool,

    /// True if the card is printed without text.
    pub textless: bool,

    /// Whether this card is a variation of another printing.
    pub variation: bool,

    /// The printing ID of the printing this card is a variation of.
    pub variation_of: Option<Guid>,

    /// The security stamp on this card, if any. One of `oval`, `triangle`, `acorn`, `arena`, or `heart`.
    pub security_stamp: Option<String>,

    /// This card’s watermark, if any.
    pub watermark: Option<String>,

    /// The date this card was previewed, a link to the preview for this card, the name of the source that previewed
    /// this card.
    pub preview: Option<HashMap<String, String>>,
}

/// # Related Card Objects
/// Cards that are closely related to other cards (because they call them by name, or generate a token, or meld, etc)
/// have a `all_parts` property that contains Related Card objects.
#[derive(Debug, Deserialize, Serialize)]
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
#[derive(Debug, Deserialize, Serialize)]
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

#[derive(Debug, Deserialize, Serialize)]
pub struct Guid(String);

#[derive(Debug, Deserialize, Serialize)]
pub struct Uri(String);

#[derive(Debug, Deserialize, Serialize)]
pub struct Colors(Vec<Color>);

#[derive(Debug, Deserialize, Serialize)]
pub struct Color(String);

#[derive(Debug, Deserialize, Serialize)]
pub struct Legality(String);

#[derive(Debug, Deserialize, Serialize)]
pub struct Date(String);
