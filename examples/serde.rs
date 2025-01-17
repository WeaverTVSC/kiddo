/// Kiddo example 2: Serde
///
/// This example extends the Serde deserialization from Example 1
/// by demonstrating serialization to/from JSON and gzipped Bincode
mod cities;

use std::error::Error;
use std::fs::File;

use elapsed::ElapsedDuration;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::time::Instant;

use kiddo::KdTree;

use serde::Deserialize;

use cities::{degrees_lat_lng_to_unit_sphere, parse_csv_file};
use kiddo::float::distance::squared_euclidean;

/// Each `CityCsvRecord` corresponds to 1 row in our city source data CSV.
///
/// Serde uses this to deserialize the CSV into a convenient format for us to work with.
#[derive(Debug, Deserialize)]
pub struct CityCsvRecord {
    #[allow(dead_code)]
    name: String,

    #[serde(rename = "latitude")]
    lat: f32,
    #[serde(rename = "longitude")]
    lng: f32,
}

impl CityCsvRecord {
    pub fn as_xyz(&self) -> [f32; 3] {
        degrees_lat_lng_to_unit_sphere(self.lat, self.lng)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Load in the cities data from the CSV and use it to populate a kd-tree, as per
    // the cities.rs example
    let start = Instant::now();
    let cities: Vec<CityCsvRecord> = parse_csv_file("./examples/geonames.csv")?;
    println!(
        "Parsed {} rows from the CSV: ({})",
        cities.len(),
        ElapsedDuration::new(start.elapsed())
    );

    let start = Instant::now();
    let mut kdtree: KdTree<f32, 3> = KdTree::with_capacity(cities.len());
    cities.iter().enumerate().for_each(|(idx, city)| {
        kdtree.add(&city.as_xyz(), idx);
    });
    println!(
        "Populated kd-tree with {} items ({})",
        kdtree.size(),
        ElapsedDuration::new(start.elapsed())
    );

    // Test query on the newly created tree
    let query = degrees_lat_lng_to_unit_sphere(52.5f32, -1.9f32);
    let (_, nearest_idx) = kdtree.nearest_one(&query, &squared_euclidean);
    let nearest = &cities[nearest_idx as usize];
    println!("\nNearest city to 52.5N, 1.9W: {:?}", nearest);

    let start = Instant::now();
    let file = File::create("./examples/geonames-tree.bincode.gz")?;
    let encoder = GzEncoder::new(file, Compression::default());
    bincode::serialize_into(encoder, &kdtree)?;
    println!(
        "Serialized kd-tree to gzipped bincode file ({})",
        ElapsedDuration::new(start.elapsed())
    );

    let start = Instant::now();
    let file = File::open("./examples/geonames-tree.bincode.gz")?;
    let decompressor = GzDecoder::new(file);
    let deserialized_tree: KdTree<f32, 3> = bincode::deserialize_from(decompressor)?;
    println!(
        "Deserialized gzipped bincode file back into a kd-tree ({})",
        ElapsedDuration::new(start.elapsed())
    );

    // Test that the deserialization worked
    let query = degrees_lat_lng_to_unit_sphere(52.5f32, -1.9f32);
    let (_, nearest_idx) = deserialized_tree.nearest_one(&query, &squared_euclidean);
    let nearest = &cities[nearest_idx as usize];
    println!("\nNearest city to 52.5N, 1.9W: {:?}", nearest);

    Ok(())
}
