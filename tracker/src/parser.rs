use crate::logwatcher::LogWatcher;
use crate::mtgadb::model::{
    AccountInfoResult, InventoryResult, InventoryUpdateResult, ParseResult, ParseResults,
    SceneChangeResult, UnknownResult, CollectionResult,
};
use crate::utils::JsonContentExtractor;
use crate::Result;
use chrono::NaiveDateTime;
use onig::Regex;

/// Reads an MTGA log file and converts its line into the output messages.
///
/// # Errors
///
/// This function will return an error it encounters a file IO error.
pub async fn watch_log(watcher: LogWatcher) -> Result<(LogWatcher, Vec<ParseResults>)> {
    let mut watcher = watcher;
    let content = String::from_utf8(watcher.read_log().await?)?;
    if content.is_empty() {
        return Ok((watcher, vec![]));
    }

    let mut current_part: Option<IntermediatePart> = None;

    let mut messages = vec![];

    let regex = Regex::new(r"^\[(.*?)\](?:(\d.+? \d.+?(?: (?:A|P)M)?)(?:$|: | (?=[a-zA-Z])))?(.*)")
        .unwrap();

    for (_, line) in content.lines().enumerate() {
        let new_part = check_new_line(&regex, line);
        match new_part {
            NewPartResult::SummarizedMessagePart(summary) => {
                // summary message should be on the second line
                assert!(current_part.is_some());
                current_part
                    .as_mut()
                    .unwrap()
                    .change_prefix(SUMMARIZED_MESSAGE_PREFIX.to_string());
                current_part.as_mut().unwrap().add_line(summary);
            }
            NewPartResult::NewPart(part) | NewPartResult::NoBracePart(part) => {
                let old = current_part.replace(part);
                if old.is_some() {
                    add_message(&mut messages, old.unwrap());
                }
            }
            NewPartResult::ContinuationPart(continuation) => {
                if current_part.is_some() {
                    current_part.as_mut().unwrap().add_line(continuation);
                }
            }
        }
    }
    if current_part.is_some() {
        add_message(&mut messages, current_part.unwrap());
    }

    Ok((watcher, messages))
}

fn check_new_line(re: &Regex, line: &str) -> NewPartResult {
    if line.starts_with("[Message summarized") {
        return NewPartResult::SummarizedMessagePart(line.to_string());
    }

    if let Some(caps) = re.captures(line) {
        let prefix = caps.at(1).map(String::from).unwrap();
        let date_str: Option<String> = caps.at(2).map(String::from);
        let content = caps.at(3).map(String::from).unwrap();
        return NewPartResult::NewPart(IntermediatePart::new(
            line.to_string(),
            prefix,
            date_str,
            content,
        ));
    }

    // no brace stuff
    let no_brace_prefixes = [
        MONO_PREFIX,
        INITIALIZING_PREFIX,
        DETAILED_LOGS_PREFIX,
        BI_ERROR_PREFIX,
    ];

    let result = no_brace_prefixes
        .iter()
        .find(|&prefix| line.starts_with(prefix))
        .map_or(
            NewPartResult::ContinuationPart(line.to_string()),
            |&prefix| {
                NewPartResult::NoBracePart(IntermediatePart::new(
                    line.to_string(),
                    String::from(prefix),
                    None,
                    line.to_string(),
                ))
            },
        );

    result
}

enum NewPartResult {
    SummarizedMessagePart(String),
    NewPart(IntermediatePart),
    NoBracePart(IntermediatePart),
    ContinuationPart(String),
}

pub struct IntermediatePart {
    first_line: String,
    prefix: String,
    date_str: Option<String>,
    content: String,
    other_lines: Vec<String>,
}

impl IntermediatePart {
    pub fn new(
        first_line: String,
        prefix: String,
        date_str: Option<String>,
        content: String,
    ) -> IntermediatePart {
        IntermediatePart {
            first_line,
            prefix,
            date_str,
            content,
            other_lines: vec![],
        }
    }
    fn prefix(&self) -> &str {
        &self.prefix
    }

    fn full_content_with_prefix(&self) -> String {
        let x = self
            .other_lines
            .iter()
            .fold(self.first_line.clone(), |init, next| {
                format!("{}\n{}", init, next)
            });

        x.trim().to_string()
    }

    fn content(&self) -> String {
        let x = self
            .other_lines
            .iter()
            .fold(self.content.clone(), |init, next| {
                format!("{}\n{}", init, next)
            });

        x.trim().to_string()
    }

    fn date(&self) -> Option<NaiveDateTime> {
        // TODO: date format is hard coded, MTGA uses CultureInfo
        let format = "%Y. %m. %d. %H:%M:%S";
        match &self.date_str {
            Some(d) => {
                let dt = chrono::NaiveDateTime::parse_from_str(d.as_str(), format);
                if dt.is_ok() {
                    Some(dt.unwrap())
                } else {
                    None
                }
            }
            None => None,
        }
    }

    fn change_prefix(&mut self, new_prefix: String) {
        self.prefix = new_prefix;
    }

    fn add_line(&mut self, line: String) {
        self.other_lines.push(line);
    }
}

fn add_message(messages: &mut Vec<ParseResults>, part: IntermediatePart) {
    let converters = [
        UnityCrossThreadLoggerParser::new(),
        MtgaDataCollectorParser::new(),
    ];

    if let Some(converter) = converters
        .iter()
        .find(|&converter| converter.can_process(&part))
    {
        let mut result = converter.parse_part(part);
        messages.append(&mut result);
    } else {
        let unknown_message = ParseResults::UnknownResult(UnknownResult::new(
            part.prefix().to_string(),
            part.date(),
            part.full_content_with_prefix(),
        ));

        messages.push(unknown_message);
    }
}

const SUMMARIZED_MESSAGE_PREFIX: &str = "Message summarized";
const MONO_PREFIX: &str = "Mono ";
const INITIALIZING_PREFIX: &str = "Initialize engine version:";
const DETAILED_LOGS_PREFIX: &str = "DETAILED LOGS:";
const BI_ERROR_PREFIX: &str = "BIError";

pub trait LogParser {
    fn get_prefix(&self) -> &str;
    fn can_process(&self, part: &IntermediatePart) -> bool;
    fn parse_part(&self, part: IntermediatePart) -> Vec<ParseResults>;
}

pub struct UnityCrossThreadLoggerParser {
    parsers: Vec<Box<dyn LogParser>>,
}

impl UnityCrossThreadLoggerParser {
    pub fn new() -> Box<dyn LogParser> {
        let parsers: Vec<Box<dyn LogParser>> = vec![SceneChangeParser::new()];

        Box::new(UnityCrossThreadLoggerParser { parsers })
    }
}

impl LogParser for UnityCrossThreadLoggerParser {
    fn get_prefix(&self) -> &str {
        "UnityCrossThreadLogger"
    }

    fn can_process(&self, part: &IntermediatePart) -> bool {
        part.prefix()
            .starts_with(<UnityCrossThreadLoggerParser as LogParser>::get_prefix(
                &self,
            ))
    }

    fn parse_part(&self, part: IntermediatePart) -> Vec<ParseResults> {
        if let Some(parser) = self.parsers.iter().find(|&p| p.can_process(&part)) {
            return parser.parse_part(part);
        } else {
            let unknown_message =
                UnknownResult::new(part.prefix().to_string(), part.date(), part.content());

            return vec![ParseResults::UnknownResult(unknown_message)];
        }
    }
}

struct SceneChangeParser {}

impl SceneChangeParser {
    pub fn new() -> Box<dyn LogParser> {
        Box::new(SceneChangeParser {})
    }
}

impl LogParser for SceneChangeParser {
    fn get_prefix(&self) -> &str {
        "Client.SceneChange"
    }

    fn can_process(&self, part: &IntermediatePart) -> bool {
        part.content()
            .starts_with(<SceneChangeParser as LogParser>::get_prefix(&self))
    }

    fn parse_part(&self, part: IntermediatePart) -> Vec<ParseResults> {
        let content = part.content();
        let json_content = JsonContentExtractor::extract(content.as_str());

        let mut result = serde_json::from_str::<SceneChangeResult>(json_content).unwrap();
        result.set_common_fields(part.prefix().to_string(), content, part.date());
        vec![ParseResults::SceneChangeResult(result)]
    }
}

pub struct MtgaDataCollectorParser {
    parser_regex: Regex,
}

impl MtgaDataCollectorParser {
    pub fn new() -> Box<dyn LogParser> {
        Box::new(MtgaDataCollectorParser {
            parser_regex: Regex::new(r"^\[(.*?)\](.*)").unwrap(),
        })
    }
}

impl LogParser for MtgaDataCollectorParser {
    fn get_prefix(&self) -> &str {
        "MTGADataCollector"
    }

    fn can_process(&self, part: &IntermediatePart) -> bool {
        part.prefix().starts_with(self.get_prefix())
    }

    fn parse_part(&self, part: IntermediatePart) -> Vec<ParseResults> {
        const ACCOUNT_INFO_PREFIX: &str = "account-info";
        const INVENTORY_UPDATE_PREFIX: &str = "inventory-update";
        const INVENTORY_PREFIX: &str = "inventory";
        const COLLECTION_PREFIX: &str = "collection";

        let mut messages = vec![];
        if let Some(caps) = self.parser_regex.captures(&part.content()) {
            let category = caps.at(1).unwrap();
            let json_content = caps.at(2).unwrap().trim();

            if category.starts_with(ACCOUNT_INFO_PREFIX) {
                let mut account_info =
                    serde_json::from_str::<AccountInfoResult>(json_content).unwrap();
                let date =
                    chrono::NaiveDateTime::parse_from_str(account_info.timestamp_str(), "%+");
                account_info.set_common_fields(
                    part.prefix().to_string(),
                    part.content(),
                    date.ok(),
                );
                messages.push(ParseResults::AccountInfoResult(account_info));
            } else if category.starts_with(INVENTORY_UPDATE_PREFIX) {
                let mut inventory_update =
                    serde_json::from_str::<InventoryUpdateResult>(json_content).unwrap();
                let date =
                    chrono::NaiveDateTime::parse_from_str(inventory_update.timestamp_str(), "%+");
                inventory_update.set_common_fields(
                    part.prefix().to_string(),
                    part.content(),
                    date.ok(),
                );
                messages.push(ParseResults::InventoryUpdateResult(inventory_update));
            } else if category.starts_with(INVENTORY_PREFIX) {
                let mut inventory = serde_json::from_str::<InventoryResult>(json_content).unwrap();
                let date = chrono::NaiveDateTime::parse_from_str(inventory.timestamp_str(), "%+");
                inventory.set_common_fields(part.prefix().to_string(), part.content(), date.ok());
                messages.push(ParseResults::InventoryResult(inventory));
            } else if category.starts_with(COLLECTION_PREFIX) {
                let mut collection = serde_json::from_str::<CollectionResult>(json_content).unwrap();
                let date = chrono::NaiveDateTime::parse_from_str(collection.timestamp_str(), "%+");
                collection.set_common_fields(part.prefix().to_string(), part.content(), date.ok());
                messages.push(ParseResults::CollectionResult(collection));
            }
        }

        messages
    }
}
