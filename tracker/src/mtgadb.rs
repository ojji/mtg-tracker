pub mod model;

use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::vec;

use rusqlite::{params, Connection, Transaction};

use model::{MtgaCard, ScryCard, TrackerCard};

pub struct MtgaDb {
    db: Connection,
}

impl MtgaDb {
    pub fn new<P>(database_path: P) -> Result<MtgaDb, Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        let db = Connection::open(database_path)?;

        Ok(MtgaDb { db })
    }

    pub fn create_database<P>(
        scryfall_cards_json_path: P,
        mtga_cards_json_path: P,
        database_path: P,
    ) -> Result<(), Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        let mut mapped_cards: HashMap<u32, (&MtgaCard, &ScryCard)> = HashMap::new();
        let scryfall_cards_path_str =
            String::from(scryfall_cards_json_path.as_ref().to_str().unwrap());
        let mtga_cards_path_str = String::from(mtga_cards_json_path.as_ref().to_str().unwrap());
        let database_path_str = String::from(database_path.as_ref().to_str().unwrap());

        let start = std::time::Instant::now();

        let mut mtga_dumped_content = Vec::new();
        File::open(mtga_cards_json_path)?.read_to_end(&mut mtga_dumped_content)?;
        let mtga_cards: Vec<MtgaCard> = serde_json::from_slice(&mtga_dumped_content)?;

        let mut scry_cards = Vec::new();
        File::open(scryfall_cards_json_path)?.read_to_end(&mut scry_cards)?;
        let scry_cards: Vec<ScryCard> = serde_json::from_slice(&scry_cards)?;

        let elapsed = start.elapsed();
        println!(
            "[{:.2?}] Scryfall and mtga database read from: {} and {}, creating card-db for the tracker... ",
            elapsed,
            scryfall_cards_path_str,
            mtga_cards_path_str
        );

        println!("There are {} mtga card objects", mtga_cards.len());

        for mtga_card in &mtga_cards {
            let scry_cards = scry_cards
                .iter()
                .filter(|scry_card| {
                    scry_card.arena_id().is_some()
                        && scry_card.arena_id().unwrap() == mtga_card.arena_id()
                        && MtgaDb::get_mtga_card_name(&mtga_cards, mtga_card)
                            == scry_card.lowercase_name()
                        && scry_card.is_available_in_arena()
                        && scry_card.lowercase_artist().is_some()
                        && scry_card.lowercase_artist().unwrap()
                            == MtgaDb::map_lowercase_mtga_artist_name_to_scry(
                                mtga_card.lowercase_artist().as_str(),
                            )
                })
                .collect::<Vec<_>>();

            match scry_cards.len() {
                0 => (),
                1 => {
                    mapped_cards.insert(mtga_card.arena_id(), (mtga_card, scry_cards[0]));
                }
                _ => {
                    let scry_card_names: Vec<String> =
                        scry_cards.iter().map(|c| c.to_string()).collect();
                    let scry_card_names = scry_card_names.join(", ");
                    panic!(
                        "warning: searching for {} ({}/{}) ({}) resulted these possible cards in the scry_db: {}",
                        MtgaDb::get_mtga_card_name(&mtga_cards, mtga_card),
                        mtga_card.set(),
                        mtga_card.collector_number(),
                        MtgaDb::map_lowercase_mtga_artist_name_to_scry(mtga_card.lowercase_artist().as_str()),
                        scry_card_names);
                }
            }
        }

        println!("There are {} elements in tracker_cards", mapped_cards.len());

        // check only collectible cards
        let mut cards_to_fix_collectible = vec![];
        for card in mtga_cards.iter().filter(|&mtga_card| {
            mtga_card.is_collectible() && !mapped_cards.contains_key(&mtga_card.arena_id())
        }) {
            cards_to_fix_collectible.push(card);
        }

        println!(
            "Trying to resolve {} collectible cards",
            cards_to_fix_collectible.len()
        );

        for mtga_card in cards_to_fix_collectible {
            match MtgaDb::get_scry_card_for_mtga_card(&mtga_cards, &scry_cards, mtga_card, false) {
                Some(scry_card) => {
                    assert!(!mapped_cards.contains_key(&mtga_card.arena_id()));
                    mapped_cards.insert(mtga_card.arena_id(), (mtga_card, scry_card));
                }
                None => {
                    let mtga_card_name = MtgaDb::get_mtga_card_name(&mtga_cards, mtga_card);
                    println!(
                        "I dont know what to do with this: {} ({}/{})",
                        mtga_card_name,
                        mtga_card.set(),
                        mtga_card.collector_number()
                    );
                }
            }
        }

        println!("There are {} elements in tracker_cards", mapped_cards.len());

        let mut remaining_cards = vec![];
        for card in mtga_cards
            .iter()
            .filter(|&mtga_card| !mapped_cards.contains_key(&mtga_card.arena_id()))
        {
            remaining_cards.push(card);
        }

        let mut remaining_cards_file = File::create("./assets/remaining_cards.txt")?;
        writeln!(
            remaining_cards_file,
            "{}",
            serde_json::to_string_pretty(&remaining_cards)?
        )?;

        println!(
            "There are {} elements in remaining_cards",
            remaining_cards.len()
        );

        let start = std::time::Instant::now();
        let mut db = Connection::open(database_path)?;
        let tx = db.transaction()?;

        MtgaDb::export_mapped_cards(&mapped_cards, &tx)?;
        MtgaDb::create_users(&tx)?;
        MtgaDb::create_user_collection(&tx)?;

        tx.commit()?;

        let elapsed = start.elapsed();

        println!(
            "[{:.2?}] Card-db has been written into {}.",
            elapsed, database_path_str
        );

        Ok(())
    }

    pub fn export_mapped_cards(
        mapped_cards: &HashMap<u32, (&MtgaCard, &ScryCard)>,
        tx: &Transaction,
    ) -> Result<(), Box<dyn Error>> {
        tx.execute("DROP TABLE IF EXISTS cards_db", [])?;
        tx.execute(
            "CREATE TABLE IF NOT EXISTS cards_db (
            'id' INTEGER PRIMARY KEY,
            'name' TEXT NOT NULL,
            'set' TEXT NOT NULL,
            'collector_number' TEXT NOT NULL,
            'scry_uri' TEXT NOT NULL,
            'arena_id' INTEGER NOT NULL,
            'image_uri' TEXT NOT NULL,
            'rarity' TEXT NOT NULL,
            'in_booster' INTEGER NOT NULL
        )",
            [],
        )?;

        tx.execute(
            "CREATE INDEX cards_db_arena_id_idx ON cards_db('arena_id')",
            [],
        )?;
        tx.execute("CREATE INDEX cards_db_set_idx ON cards_db('set')", [])?;
        tx.execute("CREATE INDEX cards_db_rarity_idx ON cards_db('rarity')", [])?;

        for (&_, &(mtga_card, scry_card)) in mapped_cards {
            let tracker_card = TrackerCard::new(mtga_card, scry_card);
            tx.execute("INSERT INTO cards_db
                ('name', 'set', 'collector_number', 'scry_uri', 'arena_id', 'image_uri', 'rarity', 'in_booster')
                    VALUES (?1, ?2, ?3, ?4, ?5,?6, ?7, ?8)",
            params![
                tracker_card.name(),
                tracker_card.set(),
                tracker_card.collector_number(),
                tracker_card.scry_uri(),
                tracker_card.arena_id(),
                tracker_card.image_uri(),
                tracker_card.rarity(),
                tracker_card.in_booster() as i32
                ])?;
        }
        Ok(())
    }

    fn create_users(tx: &Transaction) -> Result<(), Box<dyn Error>> {
        tx.execute(
            "CREATE TABLE IF NOT EXISTS users (
            'user_id' INTEGER PRIMARY KEY,
            'arena_id' TEXT NOT NULL,
            'screen_name' TEXT NOT NULL
        )",
            params![],
        )?;

        tx.execute("CREATE INDEX users_arena_id_idx ON users('arena_id')", [])?;

        Ok(())
    }

    pub fn get_card_by_id(&self, arena_id: u32) -> Option<TrackerCard> {
        TrackerCard::get_by_id(&self.db, arena_id).ok()
    }

    pub fn get_scry_card_for_mtga_card<'a>(
        mtga_cards_db: &'a [MtgaCard],
        scry_cards_db: &'a [ScryCard],
        mtga_card: &'a MtgaCard,
        was_last_resort: bool,
    ) -> Option<&'a ScryCard> {
        let mtga_card_name = MtgaDb::get_mtga_card_name(mtga_cards_db, mtga_card);

        // Try to find the card in the same set, same collector number.
        let results = scry_cards_db
            .iter()
            .filter(|&scry_card| {
                scry_card.set() == mtga_card.set()
                    && scry_card.collector_number() == mtga_card.collector_number()
            })
            .collect::<Vec<&ScryCard>>();
        match results.len() {
            0 => (), // continue
            1 => {
                // Check if the `scry_card` has an `arena_id`, sometimes these are set to a wrong id in the scry_db.
                let scry_card = results[0];

                let different_card_scry_is_referencing = {
                    if scry_card.arena_id().is_some()
                        && scry_card.arena_id().unwrap() != mtga_card.arena_id()
                    {
                        Some(
                            mtga_cards_db
                                .iter()
                                .find(|c| c.arena_id() == scry_card.arena_id().unwrap())
                                .unwrap(),
                        )
                    } else {
                        None
                    }
                };

                if mtga_card_name == scry_card.lowercase_name()
                    || (different_card_scry_is_referencing.is_some()
                        && mtga_card_name == scry_card.lowercase_name()[2..])
                {
                    return Some(scry_card);
                }
            }
            _ => {
                // try to get the `en` version of the card
                let results = results
                    .iter()
                    .filter(|c| c.lang() == "en")
                    .collect::<Vec<_>>();
                if results.len() == 1 {
                    return Some(results[0]);
                } else {
                    let scry_card_names: Vec<String> =
                        results.iter().map(|c| c.to_string()).collect();
                    let scry_card_names = scry_card_names.join(", ");
                    panic!("warning: searching for {} ({}/{}) ({}) resulted these possible cards in the scry_db: {}",
                    mtga_card_name,
                    mtga_card.set(),
                    mtga_card.collector_number(),
                    MtgaDb::map_lowercase_mtga_artist_name_to_scry(mtga_card.lowercase_artist().as_str()),
                    scry_card_names);
                }
            }
        }

        // Try to find the card in the same set, but with a different collector number.
        // Some cards is the digital varieties have inconsistent collector numbers eg. `Spider Spawning` in `ha4`
        // or `A-Young Blue Dragon // A-Sand Augury` in `hbg`.

        let results = scry_cards_db
            .iter()
            .filter(|&scry_card| {
                scry_card.lowercase_name() == mtga_card_name && scry_card.set() == mtga_card.set()
            })
            .collect::<Vec<&ScryCard>>();
        match results.len() {
            0 => (),
            1 => {
                let scry_card = results[0];
                println!(
                    "warning: {} ({}/{}) ({}) has different collector_id than {}",
                    mtga_card_name,
                    mtga_card.set(),
                    mtga_card.collector_number(),
                    MtgaDb::map_lowercase_mtga_artist_name_to_scry(
                        mtga_card.lowercase_artist().as_str()
                    ),
                    scry_card.to_string()
                );
                return Some(scry_card);
            }
            _ => {
                let scry_card_names: Vec<String> = results.iter().map(|c| c.to_string()).collect();
                let scry_card_names = scry_card_names.join(", ");
                panic!(
                    "warning: searching for {} ({}/{}) ({}) resulted these possible cards in the scry_db: {}",
                    mtga_card_name,
                    mtga_card.set(),
                    mtga_card.collector_number(),
                    MtgaDb::map_lowercase_mtga_artist_name_to_scry(mtga_card.lowercase_artist().as_str()),
                    scry_card_names
                );
            }
        }

        // last resort - try to find a card in a different set but with the same artist
        // this is a last resort because it triggers only once, to avoid infinite loop when
        // `other_mtga_card_with_same_name_and_art_id` returns 2 objects but cannot resolve a card before arriving here
        if !was_last_resort {
            println!(
                "Trying last resort for: #{}, {} ({}/{}) ({})",
                mtga_card.arena_id(),
                mtga_card_name,
                mtga_card.set(),
                mtga_card.collector_number(),
                MtgaDb::map_lowercase_mtga_artist_name_to_scry(
                    mtga_card.lowercase_artist().as_str()
                )
            );

            let results = scry_cards_db
                .iter()
                .filter(|&scry_card| {
                    mtga_card_name == scry_card.lowercase_name()
                        && scry_card.lowercase_artist().is_some()
                        && scry_card.lowercase_artist().unwrap()
                            == MtgaDb::map_lowercase_mtga_artist_name_to_scry(
                                mtga_card.lowercase_artist().as_str(),
                            )
                })
                .collect::<Vec<_>>();

            match results.len() {
                0 => {
                    println!("no results");
                }
                1 => {
                    let scry_card = results[0];
                    println!(
                        "warning: {} ({}/{}) is not {}",
                        mtga_card_name,
                        mtga_card.set(),
                        mtga_card.collector_number(),
                        scry_card.to_string()
                    );
                    return Some(scry_card);
                }
                _ => {
                    let scry_card_names: Vec<String> =
                        results.iter().map(|c| c.to_string()).collect();
                    let scry_card_names = scry_card_names.join(", ");
                    panic!("warning: searching for {} ({}/{}) ({}) resulted these possible cards in the scry_db: {}",
                                mtga_card_name,
                                mtga_card.set(),
                                mtga_card.collector_number(),
                                MtgaDb::map_lowercase_mtga_artist_name_to_scry(mtga_card.lowercase_artist().as_str()),
                                scry_card_names);
                }
            }
        }

        println!(
            "I dont know what to do with this: #{}, {} ({}/{}) ({})",
            mtga_card.arena_id(),
            mtga_card_name,
            mtga_card.set(),
            mtga_card.collector_number(),
            MtgaDb::map_lowercase_mtga_artist_name_to_scry(mtga_card.lowercase_artist().as_str())
        );
        None
    }

    fn get_mtga_card_name(mtga_cards_db: &[MtgaCard], mtga_card: &MtgaCard) -> String {
        let full_card_name = match mtga_card.linked_face_type() {
            "LinkedFace_AdventureChild" => {
                let adventure_pair = mtga_cards_db
                    .iter()
                    .find(|&card| {
                        card.linked_faces().contains(&mtga_card.arena_id())
                            && card.linked_face_type() == "LinkedFace_AdventureParent"
                    })
                    .unwrap();

                format!(r"{} // {}", &mtga_card.name(), &adventure_pair.name())
            }
            "LinkedFace_MDFC_Back" => {
                let mdfc_pair = mtga_cards_db
                    .iter()
                    .find(|&card| {
                        card.linked_faces().contains(&mtga_card.arena_id())
                            && card.linked_face_type() == "LinkedFace_MDFC_Front"
                    })
                    .unwrap();

                format!(r"{} // {}", &mtga_card.name(), &mdfc_pair.name())
            }
            "LinkedFace_DFC_Back" => {
                let dfc_pair = mtga_cards_db
                    .iter()
                    .find(|&card| {
                        card.linked_faces().contains(&mtga_card.arena_id())
                            && card.linked_face_type() == "LinkedFace_DFC_Front"
                    })
                    .unwrap();

                format!(r"{} // {}", &mtga_card.name(), &dfc_pair.name())
            }
            _ => String::from(mtga_card.name()),
        };

        full_card_name.to_lowercase()
    }

    pub fn map_lowercase_mtga_artist_name_to_scry(mtga_artist_name: &str) -> &str {
        match mtga_artist_name {
            "pascal blanche" => "pascal blanché",
            "angelo bortolini" => "ângelo bortolini",
            "l.a draws" => "la draws",
            "jihun" => "jihun lee",
            "justin & alexis hernandez" => "justin hernandez & alexis hernandez",
            "parente" => "paolo parente",
            "黒葉.k/clover.k" => "clover.k",
            "にじまあるく/nijimaarc" => "nijimaarc",
            "ライアンロロ / ryanroro" => "ryanroro",
            "蟻/ari" => "ari",
            "アジュール/azur" => "azur",
            "鴉羽凛燈 / rindo karasuba" => "rindo karasuba",
            "アマヤギ堂 / amayagido" => "amayagido",
            "桂福蔵/fukuzo katsura" => "fukuzo katsura",
            "kasia ‘kafis’ zielińska" => "kasia 'kafis' zielińska",
            "呀呀 / yaya" => "yaya",
            "寺田 克也/terada katsuya" => "terada katsuya",
            "ダイエクスト/dai-xt." => "dai-xt",
            "匈歌ハトリ / hatori kyoka" => "hatori kyoka",
            "竜徹/ryutetsu" => "ryutetsu",
            "鈴木康士 / d-suzuki" => "d-suzuki",
            "宮木一/miyaki hajime" => "miyaki hajime",
            "cardula" => "zoran cardula",
            "墨絵師「御歌頭」/ sumie okazu" => "sumie okazu",
            "えびら/ebila" => "ebila",
            "くっか / kukka" => "kukka",
            "雪代薫 / kaoru yukishiro" => "kaoru yukishiro",
            "タダ / tada" => "tada",
            "三好載克 / norikatsu miyoshi" => "norikatsu miyoshi",
            "えすてぃお/esuthio" => "esuthio",
            "タカヤマトシアキ/toshiaki takayama" => "toshiaki takayama",
            "ロルベイ/rorubei" => "rorubei",
            "yangyang / xiaji" => "yangygan & xiaji",
            "xiaji/yangyang" => "xiaji",
            "よーね/yo-ne" => "yo-ne",
            "近藤途轍/totetsu kondo" => "totetsu kondo",
            "森下直親/naochika morishita" => "naochika morishita",
            "ぴよ/piyo" => "piyo",
            "bartłomiej gaweł" => "bartlomiej gawel",
            "浮雲宇一 / uichi ukumo" => "uichi ukumo",
            "まじ/maji" => "maji",
            "v. szendrey (cashile)" => "vi szendrey (cashile)",
            "ゆのまち｡/yunomachi." => "yunomachi",
            "安達 洋介/yosuke adachi" => "yosuke adachi",
            "アオジマイコ / maiko aoji" => "maiko aoji",
            "アオジマイコ/aoji maiko" => "maiko aoji",
            "アマヤギ堂/amayagido" => "amayagido",
            "獣道/kemonomichi" => "kemonomichi",
            "ともひと/tomohito" => "tomohito",
            "ともひと / tomohito" => "tomohito",
            "七原しえ/shie nanahara" => "shie nanahara",
            "七原しえ / shie nanahara" => "shie nanahara",
            "william o’connor" => "william o'connor",
            "西野幸治 / koji nishino" => "koji nishino",
            "西野 幸治/koji nishino" => "koji nishino",
            "朋さくら / sakura tomo" => "sakura tomo",
            "士基軽太 / karuta shiki" => "karuta shiki",
            "中島 綾美 / ayami nakashima" => "ayami nakashima",
            "中島綾美/ayami nakashima" => "ayami nakashima",
            "とびはち / tobihachi" => "tobihachi",
            "山宗/sansyu" => "sansyu",
            "中坪宏太 / kota nakatsubo" => "kota nakatsubo",
            "中坪宏太/kota nakatsubo" => "kota nakatsubo",
            "マツモト ミツアキ/m.matsumoto" => "m.matsumoto",
            "百瀬寿 / hisashi momose" => "hisashi momose",
            "百瀬寿/hisashi momose" => "hisashi momose",
            "創-taro / so-taro" => "so-taro",
            "りんこ｡/rinco." => "rinco.",
            "霧糺/kiritada" => "kiritada",
            "霧糺 /  kiritada" => "kiritada",
            "霧糺 / kiritada" => "kiritada",
            "parente & brian snoddy" => "paolo parente & brian snõddy",
            "海鵜げそ/umiu geso" => "umiu geso",
            "黒井ススム/susumu kuroi" => "susumu kuroi",
            "アオガチョウ/aogachou" => "aogachou",
            "アオガチョウ / aogachou" => "aogachou",
            "jenn ravenna" => "ravenna tran",
            "村山竜大/ryota murayama" => "ryota murayama",
            "藤ちょこ/fuzichoco" => "fuzichoco",
            "伊藤未生/misei ito" => "misei ito",
            "ナブランジャ/nablange" => "nablange",
            "原哲夫/hara tetsuo" => "tetsuo hara",
            "長乃 / nagano" => "nagano",
            "武内裕季 / yuhki takeuchi" => "yuhki takeuchi",
            "吉澤舞子 / maiko yoshizawa" => "maiko yoshizawa",
            "椿春雨/tubaki halsame" => "tubaki halsame",
            "マルオユキヒロ/yukihiro maruo" => "yukihiro maruo",
            "佐久間友香 / yuka sakuma" => "yuka sakuma",
            "石川健太/ishikawa kenta" => "ishikawa kenta",
            "出利/syutsuri" => "syutsuri",
            "п猫ｒ/penekor" => "penekor",
            "あんべ よしろう/yoshiro ambe" => "yoshiro ambe",
            "萩谷薫/kaoru hagiya" => "hagiya kaoru",
            "萩谷薫 / kaoru hagiya" => "hagiya kaoru",
            "dimitar" => "dimitar marinski",
            "寿多　浩 / hiro suda" => "hiro suda",
            "石黒亜矢子 / ayako ishiguro" => "ayako ishiguro",
            "jenn ravenna tran" => "ravenna tran",
            "volkan baga" => "volkan baǵa",
            "新川洋司/yoji shinkawa" => "yoji shinkawa",
            "弥生しろ / shiro　yayoi" => "shiro yayoi",
            "風間雷太/raita kazama" => "raita kazama",
            "ゾウノセ/zounose" => "zounose",
            "村上ヒサシ/murakami hisashi" => "murakami hisashi",
            "七片藍/ai nanahira" => "ai nanahira",
            "martina fačková" => "martina fackova",
            "仙田聡/satoru senda" => "satoru senda",
            "前河悠一/maekawa yuichi" => "maekawa yuichi",
            "茶魔魔 / yaomojun" => "yaomojun",
            "brian snoddy" => "brian snõddy",
            "壱子みるく亭/ichiko milk tei" => "ichiko milk tei",
            "三好奈緒/nao miyoshi" => "nao miyoshi",
            "やまだ六角/yamada rokkaku" => "yamada rokkaku",
            "tapi岡/tapioca" => "tapioca",
            "白井秀実/shirai hidemi" => "shirai hidemi",
            "夢子 / yumeko" => "yumeko",
            "奥田まがね / magane okuda" => "magane okuda",
            "クロサワテツ/tetsu kurosawa" => "tetsu kurosawa",
            "刀 彼方/katana canata" => "katana canata",
            "加藤 綾華/kato ayaka" => "kato ayaka",
            "ヨシヤ/yoshiya" => "yoshiya",
            "小島文美 / ayami kojima" => "ayami kojima",
            "justyna gil" => "justyna dura",
            "西木あれく/areku nishiki" => "areku nishiki",
            "中村エイト/nakamura8" => "nakamura8",
            "嘉弖苅悠介 / yusuke katekari" => "yusuke katekari",
            "瞑丸イヌチヨ/inuchiyo meimaru" => "inuchiyo meimaru",
            /// FUUUUUUUUUUUUUUUUUUUUUUU "kogado studio" => ["羽山晃平", "ヨロイコウジ"],
            _ => mtga_artist_name,
        }
    }

    pub fn dump_artist_mapping_errors<P>(
        scryfall_cards_json_path: P,
        mtga_cards_json_path: P,
        output_file: P,
    ) -> Result<(), Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        let mut output_file = File::create(output_file)?;
        let mut scry_db = File::open(scryfall_cards_json_path)?;
        let mut scry_data = vec![];

        scry_db.read_to_end(&mut scry_data)?;

        let mut scry_artists = HashSet::new();

        let scry_data: Vec<ScryCard> = serde_json::from_slice(&scry_data)?;
        for scry_card in scry_data.iter() {
            if scry_card.lowercase_artist().is_some() {
                scry_artists.insert(scry_card.lowercase_artist().unwrap());
            }
        }

        let mut mtga_db = File::open(mtga_cards_json_path)?;
        let mut mtga_data = vec![];

        mtga_db.read_to_end(&mut mtga_data)?;

        let mut mtga_artists: HashMap<String, Vec<&MtgaCard>> = HashMap::new();

        let mtga_data: Vec<MtgaCard> = serde_json::from_slice(&mtga_data)?;
        for mtga_card in mtga_data.iter() {
            mtga_artists
                .entry(mtga_card.lowercase_artist())
                .or_insert(vec![])
                .push(mtga_card);
        }

        for mtga_artist in &mtga_artists {
            if scry_artists
                .iter()
                .find(|&scry_artist| {
                    scry_artist.as_str()
                        == MtgaDb::map_lowercase_mtga_artist_name_to_scry(mtga_artist.0)
                })
                .is_none()
            {
                writeln!(
                    output_file,
                    "Could not find a mapping for '{}' in the scry_db",
                    mtga_artist.0
                )?;
            }
        }

        writeln!(
            output_file,
            "\n//---------Scry---------//\n{}",
            serde_json::to_string_pretty(&scry_artists)?
        )?;
        writeln!(
            output_file,
            "\n//---------Mtga---------//\n{}",
            serde_json::to_string_pretty(&mtga_artists)?
        )?;

        Ok(())
    }

    pub fn get_user_session(
        &self,
        account_info: model::AccountInfoData,
    ) -> Result<UserSession, Box<dyn Error>> {
        let mut stmt = self.db.prepare(
            "
            SELECT users.'user_id', users.'arena_id', users.'screen_name'
            FROM users
            WHERE users.'arena_id' = ?1",
        )?;

        let mut results = stmt.query(params![account_info.user_id])?;

        match results.next()? {
            Some(user) => {
                return Ok(UserSession {
                    user_id: user.get(0)?,
                    arena_id: user.get(1)?,
                    screen_name: user.get(2)?,
                })
            }
            None => {
                self.db.execute(
                    "
                INSERT INTO users ('arena_id', 'screen_name')
                VALUES (?1, ?2)",
                    params![account_info.user_id, account_info.screen_name],
                )?;

                return self.get_user_session(account_info);
            }
        }
    }

    fn create_user_collection(tx: &Transaction) -> Result<(), Box<dyn Error>> {
        tx.execute(
            "CREATE TABLE IF NOT EXISTS user_collections (
            'collection_id' INTEGER PRIMARY KEY,
            'user_id' INTEGER NOT NULL,
            'timestamp' TEXT NOT NULL,
            'collection_data' BLOB NOT NULL
        )",
            params![],
        )?;

        tx.execute(
            "CREATE INDEX user_collections_user_id_idx ON user_collections('user_id')",
            [],
        )?;

        Ok(())
    }

    pub fn update_user_collection(
        &self,
        current_user: &UserSession,
        collection_update: model::CollectionEvent,
    ) -> Result<(), Box<dyn Error>> {
        self.db.execute(
            "
            INSERT INTO user_collections
                ('user_id', 'timestamp', 'collection_data')
            VALUES (?1, ?2, ?3)",
            params![
            current_user.user_id,
            collection_update.timestamp,
            serde_json::to_string(&collection_update.attachment)?
            ],
        )?;

        Ok(())
    }
}

/// Type representing a user in the tracker client
pub struct UserSession {
    /// The collector user id
    user_id: u32,
    /// The user id in the MTGA client
    arena_id: String,
    /// The user's name in the MTGA client
    screen_name: String,
}

impl UserSession {
    pub fn screen_name(&self) -> &str {
        &self.screen_name
    }
}
