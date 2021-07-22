use chrono::NaiveDate;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::prelude::*;

#[derive(Debug)]
struct Post {
    meta: Frontmatter,
    body: String,
}

#[derive(Debug, Serialize)]
struct Frontmatter {
    date: NaiveDate,
    title: String,
    slug: String,
    category: Option<String>,
    tags: Vec<String>,
}

fn main() {
    let contents =
        fs::read_to_string("dump/data.json").expect("Something went wrong reading the file");
    let data: Value = serde_json::from_str(&contents).expect("Couldn't parse the json");

    let tags = get_lookup_by_content_type_id("tag", &data);
    let categories = get_lookup_by_content_type_id("5KMiN6YPvi42icqAUQMCQe", &data);
    let posts = get_posts(&data, tags, categories);

    for (id, post) in posts {
        save(post);
    }

    println!("done!");
}

fn save(post: Post) {
    let mut buffer = String::new();
    buffer.push_str(&serde_yaml::to_string(&post.meta).unwrap());
    buffer.push_str("---\n\n");
    buffer.push_str(&post.body);

    let mut file = File::create(format!("blog/{}.md", post.meta.slug)).unwrap();
    file.write_all(buffer.as_bytes()).unwrap();
}

fn get_posts(
    data: &Value,
    tags: HashMap<String, String>,
    categories: HashMap<String, String>,
) -> HashMap<String, Post> {
    let kind = "2wKn6yEnZewu2SCCkus4as";

    let get_field = |key, entry: &Value| entry["fields"][key]["en-US"].as_str().unwrap().to_owned();
    let get_category_str = |entry: &Value| {
        let id = entry["fields"]["category"]["en-US"]["sys"]["id"]
            .as_str()
            .unwrap();
        categories.get(id).cloned()
    };

    let get_tag_str = |entry: &Value| {
        entry["fields"]["tags"]["en-US"]
            .as_array()
            .map(|tt| {
                tt.iter()
                    .filter_map(|t| {
                        let id = t["sys"]["id"].as_str().unwrap();
                        tags.get(id).cloned()
                    })
                    .collect()
            })
            .unwrap_or_default()
    };

    let get_date = |entry: &Value| {
        let date = get_field("date", entry);
        let date = date.split("T").next().unwrap();
        date.parse::<NaiveDate>().unwrap()
    };

    data["entries"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|entry| entry["sys"]["contentType"]["sys"]["id"].as_str().unwrap() == kind)
        .map(|entry| {
            (
                entry["sys"]["id"].as_str().unwrap().to_owned(),
                Post {
                    meta: Frontmatter {
                        date: get_date(entry),
                        title: get_field("title", entry),
                        slug: get_field("slug", entry),
                        category: get_category_str(entry),
                        tags: get_tag_str(entry),
                    },
                    body: get_field("body", entry),
                },
            )
        })
        .collect()
}

fn get_lookup_by_content_type_id(kind: &str, data: &Value) -> HashMap<String, String> {
    data["entries"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|entry| entry["sys"]["contentType"]["sys"]["id"].as_str().unwrap() == kind)
        .map(|entry| {
            (
                entry["sys"]["id"].as_str().unwrap().to_owned(),
                entry["fields"]["title"]["en-US"]
                    .as_str()
                    .unwrap()
                    .to_owned(),
            )
        })
        .collect()
}
