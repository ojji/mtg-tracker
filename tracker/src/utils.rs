pub struct JsonContentExtractor {}

impl JsonContentExtractor {
    pub fn extract(content: &str) -> &str {
        let arr_start_idx = content.find('[');
        let obj_start_idx = content.find('{');

        let (start_idx, end_char) = {
            if arr_start_idx.is_none() {
                (obj_start_idx.unwrap(), '}')
            } else if obj_start_idx.is_none() {
                (arr_start_idx.unwrap(), ']')
            } else if arr_start_idx.unwrap() < obj_start_idx.unwrap() {
                (arr_start_idx.unwrap(), ']')
            } else {
                (obj_start_idx.unwrap(), '}')
            }
        };

        let end_idx = content.find(end_char).unwrap();

        &content[start_idx..=end_idx]
    }
}
