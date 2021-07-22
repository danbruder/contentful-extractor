use chrono::NaiveDate;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::prelude::*;

fn main() {
    let contents =
        fs::read_to_string("contentful-data.json").expect("Something went wrong reading the file");
    let data: Value = serde_json::from_str(&contents).expect("Couldn't parse the json");

    let tag_content_type_id = "tag";
    let category_content_type_id = "5KMiN6YPvi42icqAUQMCQe";
    let post_content_type_id = "2wKn6yEnZewu2SCCkus4as";

    let tags = get_lookup_by_content_type_id(tag_content_type_id, &data);
    let categories = get_lookup_by_content_type_id(category_content_type_id, &data);
    let posts = get_posts(post_content_type_id, data, tags, categories);

    let dir_prefix = "blog";
    for (_, post) in posts {
        save(dir_prefix, post);
    }

    println!("Finished");
}

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

/// Save the post as a markdown file. Need to manually create a blog dir first.
fn save(dir_prefix: &str, post: Post) {
    let mut buffer = String::new();
    buffer.push_str(&serde_yaml::to_string(&post.meta).unwrap());
    buffer.push_str("---\n\n");
    buffer.push_str(&post.body);

    let mut file = File::create(format!("{}/{}.md", dir_prefix, post.meta.slug)).unwrap();
    file.write_all(buffer.as_bytes()).unwrap();
}

/// Extract the posts from the json payload
fn get_posts(
    post_content_type_id: &str,
    data: Value,
    tags: HashMap<String, String>,
    categories: HashMap<String, String>,
) -> HashMap<String, Post> {
    data["entries"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|entry| {
            entry["sys"]["contentType"]["sys"]["id"].as_str().unwrap() == post_content_type_id
        })
        .map(|entry| {
            (
                entry["sys"]["id"].as_str().unwrap().to_owned(),
                Post {
                    meta: Frontmatter {
                        date: get_date(entry),
                        title: get_field("title", entry),
                        slug: get_field("slug", entry),
                        category: get_category(entry, &categories),
                        tags: get_tags(entry, &tags),
                    },
                    body: get_field("body", entry),
                },
            )
        })
        .collect()
}

/// Create a map from id to entry by content type
fn get_lookup_by_content_type_id(content_type_id: &str, data: &Value) -> HashMap<String, String> {
    data["entries"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|entry| {
            entry["sys"]["contentType"]["sys"]["id"].as_str().unwrap() == content_type_id
        })
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

/// Extract a single field by key
fn get_field(key: &str, entry: &Value) -> String {
    entry["fields"][key]["en-US"].as_str().unwrap().to_owned()
}

/// Extract the date field
fn get_date(entry: &Value) -> NaiveDate {
    let date = get_field("date", entry);
    let date = date.split("T").next().unwrap();
    date.parse::<NaiveDate>().unwrap()
}

/// Extract the tags
fn get_tags(entry: &Value, tags: &HashMap<String, String>) -> Vec<String> {
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
}

/// Extract the category
fn get_category(entry: &Value, categories: &HashMap<String, String>) -> Option<String> {
    let id = entry["fields"]["category"]["en-US"]["sys"]["id"]
        .as_str()
        .unwrap();
    categories.get(id).cloned()
}
