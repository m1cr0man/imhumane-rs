use std::{sync::{Arc, RwLock, Mutex}, path::{PathBuf, Path}, io::Cursor, time::{Duration, Instant}, collections::HashMap};
use snafu::prelude::*;
use rand::prelude::*;
use uuid::Uuid;
use image::{GenericImage, ImageFormat, imageops::{FilterType, self}, RgbImage, Rgb};

use super::{challenge::Challenge, collection::Collection, error::*};

type Result<T, E = Error> = std::result::Result<T, E>;

const GAP_PX: u32 = 8;
const IMG_SIZE_PX: u32 = 96;

#[derive(Debug, Clone)]
pub struct ImHumane {
    queue: Arc<deadqueue::resizable::Queue<Challenge>>,
    collections: Arc<RwLock<Vec<Collection>>>,
    answers: Arc<Mutex<HashMap<String, u32>>>,
}

impl ImHumane {
    pub fn new(buffer_size: usize) -> Self {
        Self {
            queue: Arc::new(deadqueue::resizable::Queue::new(buffer_size)),
            collections: Arc::new(RwLock::new(Vec::new())),
            answers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn try_get_challenge(&self) -> Option<Challenge> {
        self.queue.try_pop()
    }

    pub async fn get_challenge(&self) -> Challenge {
        self.queue.pop().await
    }

    pub fn check_answer(&self, challenge_id: String, answer: u32) -> bool {
        if let Some(correct_answer) = self.answers.lock().unwrap().remove(&challenge_id) {
            println!("Recived answer {} for {}. Expected {}", answer, &challenge_id, correct_answer);
            return correct_answer == answer
        }
        return false
    }

    pub fn run_generator(&self, handle: tokio::runtime::Handle) {
        // This function relies on the limited capacity of the queue
        // to limit the number of challenges generated.
        loop {
            let start = Instant::now();
            match self.generate() {
                Ok(challenge) => {
                    self.answers.lock().unwrap().insert(challenge.id.clone(), challenge.answer.clone());
                    println!("Generated in {}ms. {}", start.elapsed().as_millis(), challenge);
                    handle.block_on(self.queue.push(challenge));
                },
                Err(err) => {
                    println!("Failed to generate challenge: {:#}", err);
                    std::thread::sleep(Duration::from_secs(1));
                },
            }
        }
    }

    fn generate_image(&self, images: Vec<&(&PathBuf, u32)>) -> Result<Vec<u8>> {
        // Assume a square grid
        let grid_xy = f32::ceil(f32::sqrt(images.len() as f32)) as u32;
        let img_area = IMG_SIZE_PX + GAP_PX;
        let dimensions = (grid_xy * img_area) + GAP_PX;
        let mut imgbuf = RgbImage::new(dimensions, dimensions);
        imageops::vertical_gradient(&mut imgbuf, &Rgb([180, 180, 200]), &Rgb([220, 240, 220]));

        let mut i = 0;
        for img in images {
            println!("Inserting {}", img.0.display());
            let test_img = image::open(img.0).context(OpenImageSnafu::from(img.0))?;
            let test_img = test_img.resize(IMG_SIZE_PX, IMG_SIZE_PX, FilterType::Triangle);
            imgbuf.copy_from(test_img.as_rgb8().unwrap(), GAP_PX + (img_area * (i % grid_xy)), GAP_PX + (img_area * (i / grid_xy))).context(GenerateImageSnafu {})?;
            i += 1;
        }

        let mut data = Vec::new();

        let mut outbuf = Cursor::new(&mut data);
        println!("Generating image");
        imgbuf.write_to(&mut outbuf, ImageFormat::Jpeg).context(GenerateImageSnafu {})?;

        Ok(data)
    }

    pub fn generate(&self) -> Result<Challenge> {
        // Clone to free the lock
        let collections = self.collections.read().unwrap().clone();

        if collections.len() < 2 {
            return (InsufficientCollectionsSnafu {}).fail();
        }

        let mut rng = thread_rng();

        let num_collections = rng.gen_range(2..=std::cmp::min(collections.len(), 5));

        let mut sample = collections.choose_multiple(&mut rng, num_collections);

        // The first entry of the sample will be our "correct" collection
        let correct = sample.next().context(InsufficientCollectionsSnafu {})?;

        // Weight correct answers with (num_collections)
        let mut images: Vec<_> = correct.images.iter().map(|img| (img, num_collections as u32)).collect();

        // Weight incorrect answers with 1
        for collection in sample {
            collection.images.iter().for_each(|img| images.push((img, 1)));
        }

        let question_images: Vec<_> = images.choose_multiple_weighted(&mut rng, 9, |(_,v)| *v).unwrap().collect();

        let mut answer: u32 = 0x0;
        let mut i = 0;
        for (_, weight) in question_images.iter() {
            if *weight == num_collections as u32 {
                answer |= 0x1 << i;
            }
            i += 1;
        }

        Ok(Challenge {
            id: Uuid::new_v4().to_string(),
            image: self.generate_image(question_images)?,
            topic: correct.name.clone(),
            answer,
        })
    }

    pub fn scan_for_collections(&self, root: &Path) -> Result<()> {
        let mut collections= Vec::new();

        for entry in root.read_dir().context(ScanSnafu::from(root))? {
            let entry = entry.context(ScanSnafu::from(root))?;
            let path = entry.path();
            println!("{}", path.display());

            let ftype =  entry.file_type().context(ScanSnafu::from(path.as_path()))?;

            // New collection
            if ftype.is_dir() {

                // Scan for images
                let mut images = Vec::new();
                for image in path.read_dir().context(ScanSnafu::from(path.as_path()))? {
                    let image = image.context(ScanSnafu::from(path.as_path()))?;
                    let img_path = image.path();

                    if img_path.is_file() {
                        println!("{}", img_path.display());
                        images.push(img_path);
                    }
                }

                if images.len() == 0 {
                    continue
                }

                // into_string is a weird function. Err is an OsString
                let name = match entry.file_name().into_string() {
                    Ok(name) => name,
                    Err(_) => {
                        return CollectionNameSnafu::from(path.as_path()).fail();
                    },
                };

                collections.push(Collection {
                    // path,
                    name,
                    images,
                });
            }
        }

        let mut existing_collections = self.collections.write().unwrap();
        existing_collections.clear();
        existing_collections.append(&mut collections);

        Ok(())
    }

}
