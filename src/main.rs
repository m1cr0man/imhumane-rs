use std::{sync::{Arc, RwLock}, collections::HashMap, net::{Ipv4Addr, SocketAddr}, time::Duration, path::{PathBuf, Path}, io::{Cursor, Read}};
use image::{ImageBuffer, GenericImage, ImageError, ImageFormat, imageops::{FilterType, self}, Rgba, RgbImage, Rgb, RgbaImage};
use snafu::prelude::*;
use tower::ServiceBuilder;
use tower_http::{
    timeout::TimeoutLayer,
    trace::TraceLayer,
    ServiceBuilderExt,
};
use base64::prelude::*;
use axum::{
    routing::get,
    Router, body::Bytes, extract, http::StatusCode, response::{Html, IntoResponse},
};
use uuid::Uuid;
use rand::prelude::*;

const IMG_COUNT_XY: u32 = 3;
const GAP_PX: u32 = 8;
const IMG_SIZE_PX: u32 = 96;

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(display("Could not scan for collections in {path}"))]
    Scan { path: String, source: std::io::Error },
    #[snafu(display("Could not convert a folder name to a string: {path}"))]
    CollectionName { path: String },
    #[snafu(display("Could not read image {path}"))]
    ReadImage { path: String, source: std::io::Error },
    #[snafu(display("Could not lock state for key {key}"))]
    StateLock { key: String },
    #[snafu(display("Insufficient collections for a valid question"))]
    InsufficientCollections,
    #[snafu(display("Failed to generate collage image: {source}"))]
    GenerateImage { source: ImageError },
    #[snafu(display("Failed to open image {path}"))]
    OpenImage { path: String, source: ImageError },
}

impl From<&Path> for ScanSnafu<String> {
    fn from(value: &Path) -> Self {
        Self {
            path: value.display().to_string()
        }
    }
}

impl From<&Path> for CollectionNameSnafu<String> {
    fn from(value: &Path) -> Self {
        Self {
            path: value.display().to_string()
        }
    }
}

impl From<&PathBuf> for OpenImageSnafu<String> {
    fn from(value: &PathBuf) -> Self {
        Self {
            path: value.display().to_string()
        }
    }
}

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Clone, Debug)]
struct ImageCollection {
    path: PathBuf,
    name: String,
    images: Vec<PathBuf>,
}

#[derive(Clone, Debug)]
struct AppState {
    answers: Arc<RwLock<HashMap<String, u32>>>,
    collections: Arc<RwLock<Vec<ImageCollection>>>,
}

fn generate_image(images: Vec<&(&PathBuf, u32)>) -> Result<Vec<u8>> {
    // Assume a square grid
    let grid_xy = f32::ceil(f32::sqrt(images.len() as f32)) as u32;
    let img_area = IMG_SIZE_PX + GAP_PX;
    let dimensions = (grid_xy * img_area) + GAP_PX;
    let mut imgbuf = RgbImage::new(dimensions, dimensions);
    imageops::vertical_gradient(&mut imgbuf, &Rgb([180, 180, 200]), &Rgb([220, 240, 220]));

    let mut i = 0;
    for img in images {
        let test_img = image::open(img.0).context(OpenImageSnafu::from(img.0))?;
        let test_img = test_img.resize(IMG_SIZE_PX, IMG_SIZE_PX, FilterType::Triangle);
        imgbuf.copy_from(test_img.as_rgb8().unwrap(), GAP_PX + (img_area * (i % grid_xy)), GAP_PX + (img_area * (i / grid_xy))).context(GenerateImageSnafu {})?;
        i += 1;
    }

    let mut data = Vec::new();

    let mut outbuf = Cursor::new(&mut data);
    imgbuf.write_to(&mut outbuf, ImageFormat::Jpeg).context(GenerateImageSnafu {})?;

    // let mut out = Vec::new();
    // outbuf.read_to_end(&mut out).unwrap();

    Ok(data)
}

fn generate(state: extract::State<AppState>) -> Result<(String, Vec<u8>)> {
    // Clone to free the lock
    let collections = state.collections.read().unwrap().clone();

    if collections.len() < 2 {
        return (InsufficientCollectionsSnafu {}).fail();
    }

    let mut rng = thread_rng();

    let num_collections = rng.gen_range(2..=std::cmp::min(collections.len(), 5));

    let mut sample = collections.choose_multiple(&mut rng, num_collections);

    // The first entry of the sample will be our "correct" collection
    let correct = sample.next().context(InsufficientCollectionsSnafu {})?;
    let question = format!("Select all images containing {}", correct.name);

    // Weight correct answers with (num_collections)
    let mut images: Vec<_> = correct.images.iter().map(|img| (img, num_collections as u32)).collect();

    // Weight incorrect answers with 1
    for collection in sample {
        collection.images.iter().for_each(|img| images.push((img, 1)));
    }

    let question_images: Vec<_> = images.choose_multiple_weighted(&mut rng, 9, |(_,v)| *v).unwrap().collect();

    let mut right_answer: u32 = 0x0;
    for (_, weight) in question_images.iter() {
        if *weight == num_collections as u32 {
            right_answer |= 0x1
        }
        right_answer = right_answer << 1;
    }

    let collage = generate_image(question_images)?;

    let id = Uuid::new_v4();

    // Keep the lock for as little time as possible
    {
        let mut answers = state.answers.write().unwrap();
        answers.insert(id.to_string(), right_answer);
    }

    Ok((question, collage))
}

async fn get_test(state: extract::State<AppState>) -> impl IntoResponse {
    match generate(state) {
        Err(err) => Html(
            format!(r#"
            <!DOCTYPE html>
            <html>
                <head><title>I'm Humane</title></head>
                <body>
                    <h1>Oops</h1>
                    <pre>{:#}</pre>
                </body>
            </html>
            "#, err)
        ).into_response(),
        Ok((question, collage)) => Html(
            format!(r#"
            <!DOCTYPE html>
            <html>
                <head><title>I'm Humane</title></head>
                <body>
                    <h1>{}</h1>
                    <img src="data:image/jpeg;base64, {}" alt="Test image">
                </body>
            </html>
            "#, question, BASE64_STANDARD.encode(collage))
        ).into_response(),
    }
}

async fn check(path: extract::Path<String>, state: extract::State<AppState>) {

}

fn scan_for_collections(root: &Path) -> Result<Vec<ImageCollection>> {
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

            collections.push(ImageCollection {
                path,
                name,
                images,
            });
        }
    }

    Ok(collections)
}

fn app() -> Router {
    let collections = scan_for_collections(Path::new("images")).unwrap();
    println!("Found {} collections", collections.len());
    let state = AppState {
        answers: Arc::new(RwLock::new(HashMap::new())),
        collections: Arc::new(RwLock::new(collections)),
    };

    let middleware = ServiceBuilder::new()
        .layer(
            TraceLayer::new_for_http()
                .on_body_chunk(|chunk: &Bytes, latency: Duration, _: &tracing::Span| {
                    tracing::trace!(size_bytes = chunk.len(), layency = ?latency, "sending body chunk")
                    // TODO https://github.com/tower-rs/tower-http/blob/master/examples/axum-key-value-store/src/main.rs#L55
                })
        )
        .layer(TimeoutLayer::new(Duration::from_secs(10)))
        .map_response_body(axum::body::boxed);

    Router::new().route("/", get(get_test).layer(middleware).with_state(state))
}

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 3000));
    tracing::info!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app().into_make_service())
        .await
        .unwrap();
}
