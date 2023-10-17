use crate::{mtgadb::MtgaDb, Result};
use async_std::{
    fs::File,
    io::{ReadExt, WriteExt},
};
use iced::widget::image;

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

pub struct ImageLoader {}

impl ImageLoader {
    pub async fn load_image(arena_id: u32, db: MtgaDb) -> Result<(u32, image::Handle)> {
        let file_path = format!("assets/cards/{arena_id}.jpg");
        if let Ok(mut image_fd) = File::open(file_path.clone()).await {
            let mut image_data = vec![];
            let _ = image_fd.read_to_end(&mut image_data).await?;
            Ok((arena_id, image::Handle::from_memory(image_data)))
        } else {
            let uri = db.get_scry_image_uri(arena_id)?;
            let image_body = reqwest::get(uri.clone()).await?.bytes().await?;
            if let Ok(mut file_to_write) = File::create(file_path.clone()).await {
                file_to_write.write_all(image_body.as_ref()).await?;
                Ok((arena_id, image::Handle::from_memory(image_body.clone())))
            } else {
                Ok((
                    arena_id,
                    image::Handle::from_path("assets/cards/no_image_available.png"),
                ))
            }
        }
    }
}
