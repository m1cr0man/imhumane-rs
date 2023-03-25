use image::{
    imageops::{self, FilterType},
    DynamicImage, GenericImage, ImageFormat, Rgb, RgbImage,
};
use rand::prelude::*;
use snafu::prelude::*;
use std::{
    collections::{HashMap, HashSet},
    ffi::OsString,
    io::{BufReader, Cursor, Seek},
    path::{Path, PathBuf},
    sync::{Mutex, RwLock},
    time::{Duration, Instant},
};
use uuid::Uuid;

use super::{challenge::Challenge, collection::Collection, error::*, locked_file::LockedFile};

type Result<T, E = Error> = std::result::Result<T, E>;

const GAP_PX: u32 = 8;
const IMG_SIZE_PX: u32 = 96;
const THUMBNAIL_PREFIX: &str = &".thumbnail.";

#[derive(Debug)]
pub struct ImHumane {
    queue: deadqueue::resizable::Queue<Challenge>,
    thumbnail_queue: deadqueue::unlimited::Queue<PathBuf>,
    collections: RwLock<Vec<Collection>>,
    answers: Mutex<HashMap<String, u32>>,
    validated_tokens: Mutex<HashSet<String>>,
}

fn get_thumbnail_path(img_path: &PathBuf) -> PathBuf {
    // Fancy filename gen to avoid an unnecessary conversion to str
    let mut thumbnail = OsString::from(THUMBNAIL_PREFIX);
    thumbnail.push(img_path.file_stem().unwrap());
    thumbnail.push(".jpg");
    img_path.with_file_name(thumbnail)
}

impl ImHumane {
    pub fn new(buffer_size: usize) -> Self {
        Self {
            queue: deadqueue::resizable::Queue::new(buffer_size),
            thumbnail_queue: deadqueue::unlimited::Queue::new(),
            collections: RwLock::new(Vec::new()),
            answers: Mutex::new(HashMap::new()),
            validated_tokens: Mutex::new(HashSet::new()),
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
            println!(
                "Recived answer {} for {}. Expected {}",
                answer, &challenge_id, correct_answer
            );
            if correct_answer == answer {
                self.validated_tokens.lock().unwrap().insert(challenge_id);
                return true;
            }
        }
        return false;
    }

    pub fn check_token(&self, challenge_id: String) -> bool {
        self.validated_tokens.lock().unwrap().remove(&challenge_id)
    }

    pub fn run_generator(&self, handle: tokio::runtime::Handle) {
        // This function relies on the limited capacity of the queue
        // to limit the number of challenges generated.
        loop {
            let start = Instant::now();
            match self.generate() {
                Ok(challenge) => {
                    self.answers
                        .lock()
                        .unwrap()
                        .insert(challenge.id.clone(), challenge.answer.clone());
                    println!(
                        "Generated in {}ms. {}",
                        start.elapsed().as_millis(),
                        challenge
                    );

                    // If queue would block, take a moment to generate a thumbnail
                    while self.queue.is_full() {
                        if let Some(img_path) = self.thumbnail_queue.try_pop() {
                            println!(
                                "Taking a moment to generate a thumbnail ({})",
                                img_path.display()
                            );
                            match self.get_thumbnail(&img_path) {
                                Err(err) => println!(
                                    "Failed to generate thumbnail for {}: {:?}",
                                    img_path.display(),
                                    err
                                ),
                                _ => {}
                            }
                        } else {
                            break;
                        }
                    }

                    handle.block_on(self.queue.push(challenge));
                }
                Err(err) => {
                    println!("Failed to generate challenge: {:#}", err);
                    std::thread::sleep(Duration::from_secs(1));
                }
            }
        }
    }

    fn get_thumbnail(&self, img_path: &PathBuf) -> Result<DynamicImage> {
        // Need to make sure that only one thread is generating the content of this thumbnail at a time.
        let thumb_err = OpenThumbnailSnafu {
            path: img_path.as_path(),
        };
        let thumb_path = get_thumbnail_path(img_path);
        let locked_file = LockedFile::open_rw_no_truncate(thumb_path.clone()).context(thumb_err)?;
        let mut file = &locked_file.file;

        // Check if the written data is a valid thumbnail
        if file.metadata().context(thumb_err)?.len() > 0 {
            let reader = BufReader::new(file);
            let fmt = ImageFormat::Jpeg;
            let img = image::load(reader, fmt).context(OpenImageSnafu::from(img_path))?;
            if img.width() == IMG_SIZE_PX && img.height() == IMG_SIZE_PX {
                println!("Reusing saved thumbnail for {}", thumb_path.display());
                return Ok(img);
            }
        }

        // Otherwise create a new one
        println!("Generating thumbnail for {}", img_path.display());
        file.seek(std::io::SeekFrom::Start(0)).context(thumb_err)?;
        file.set_len(0).context(thumb_err)?;

        let orig_img = image::open(img_path.clone()).context(OpenImageSnafu::from(img_path))?;
        let orig_img = orig_img.resize(IMG_SIZE_PX, IMG_SIZE_PX, FilterType::Triangle);
        orig_img
            .save_with_format(thumb_path, ImageFormat::Jpeg)
            .context(GenerateImageSnafu {})?;

        Ok(orig_img)
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
            let test_img = self.get_thumbnail(&img.0)?;
            imgbuf
                .copy_from(
                    test_img.as_rgb8().unwrap(),
                    GAP_PX + (img_area * (i % grid_xy)),
                    GAP_PX + (img_area * (i / grid_xy)),
                )
                .context(GenerateImageSnafu {})?;
            i += 1;
        }

        let mut data = Vec::new();

        let mut outbuf = Cursor::new(&mut data);
        println!("Generating image");
        imgbuf
            .write_to(&mut outbuf, ImageFormat::Jpeg)
            .context(GenerateImageSnafu {})?;

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
        let mut images: Vec<_> = correct
            .images
            .iter()
            .map(|img| (img, num_collections as u32))
            .collect();

        // Weight incorrect answers with 1
        for collection in sample {
            collection
                .images
                .iter()
                .for_each(|img| images.push((img, 1)));
        }

        let question_images: Vec<_> = images
            .choose_multiple_weighted(&mut rng, 9, |(_, v)| *v)
            .unwrap()
            .collect();

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
        let mut collections = Vec::new();

        for entry in root.read_dir().context(ScanSnafu::from(root))? {
            let entry = entry.context(ScanSnafu::from(root))?;
            let path = entry.path();
            println!("{}", path.display());

            let ftype = entry.file_type().context(ScanSnafu::from(path.as_path()))?;

            // New collection
            if ftype.is_dir() {
                // Scan for images
                let mut images = Vec::new();
                for image in path.read_dir().context(ScanSnafu::from(path.as_path()))? {
                    let image = image.context(ScanSnafu::from(path.as_path()))?;
                    let img_path = image.path();

                    if img_path.is_file()
                        && !img_path
                            .file_name()
                            .unwrap()
                            .to_string_lossy()
                            .starts_with(THUMBNAIL_PREFIX)
                    {
                        // Check if this image needs a thumbnail generated
                        let thumbnail = get_thumbnail_path(&img_path);
                        if !thumbnail.exists() {
                            println!("{} added to thumbnail queue", img_path.display());
                            self.thumbnail_queue.push(img_path.clone());
                        }

                        println!("{}", img_path.display());
                        images.push(img_path);
                    }
                }

                if images.len() == 0 {
                    continue;
                }

                // into_string is a weird function. Err is an OsString
                let name = match entry.file_name().into_string() {
                    Ok(name) => name,
                    Err(_) => {
                        return CollectionNameSnafu::from(path.as_path()).fail();
                    }
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
