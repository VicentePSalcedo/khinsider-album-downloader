use std::fs;
use std::fs::File;
use std::io;
use std::path::Path;
use std::thread;

extern crate threadpool;
use threadpool::ThreadPool;

extern crate reqwest;
use reqwest::blocking::{Client,get};

extern crate clap;
use clap::Parser;

extern crate scraper;
use scraper::{Html, Selector};

extern crate url;
use url::Url;

extern crate regex;
use regex::Regex;

#[derive(Parser, Debug)]
struct Args {
    url: String,
    target_dir: String,
    file_type: String,
}

#[derive(Debug)]
struct Links {
    url: Option<String>,
}

fn main() {
    let website = "https://downloads.khinsider.com";

    let mut album = String::new();
    let args = Args::parse();
    let album_url = &args.url;
    let parsed_url = Url::parse(album_url);
    match parsed_url {
        Ok(parsed_url) => {
            for segment in parsed_url.path_segments().unwrap() {
                album = album + "/" + segment;
            }
        }
        Err(err) => {
            println!("Double check the url. Error: {}",err);
            std::process::exit(1);
        }
    }

    let url_str = website.to_owned() + &album;
    let target_dir = &args.target_dir;
    let target_dir_path = Path::new(&target_dir);
    if !target_dir_path.exists() {
        println!("Error: \"target_dir\" does not exist");
        std::process::exit(1);
    }

    let file_type = &args.file_type;
    if !(file_type == "flac" || file_type == "mp3") {
        println!("Error: Invalid \"file_type\". Use \"flac\" or \"mp3\"");
        std::process::exit(1);
    }

    let website_html = get_html(&url_str);
    let download_page_links: Vec<Links> =
        get_element_links(&website_html, "td.playlistDownloadSong");
    if download_page_links.is_empty() {
        println!("Error: No song download links found in that \"album_url\"");
        std::process::exit(1);
    }

    let title_element = get_tag(&website_html, "h2");
    let title_element_txt = get_text(&title_element[0], "h2");
    let album_dir = scrub_windows_dir_name(&title_element_txt) + "/";
    let dir_str = target_dir.to_owned() + "/" + &album_dir;

    println!("Album: {:?}", title_element_txt);
    match fs::create_dir(dir_str) {
        Ok(_) => (),
        Err(err) => {
            // Handle the error
            if err.kind() == io::ErrorKind::AlreadyExists {
            } else {
                // Handle other errors (e.g., permission issues)
                println!("Error creating directory: {}", err);
            }
        }
    }

    let index;
    let file_extension;
    if file_type == "mp3" {
        index = 3;
        file_extension = ".mp3";
    } else {
        index = 4;
        file_extension = ".flac";
    }

    let pool = ThreadPool::new(thread::available_parallelism().unwrap().get());
    for download_button in download_page_links {
        let url = website.to_owned() + &download_button.url.unwrap();
        let download_page = get_html(&url);
        let song_links = get_element_links(&download_page, "p");
        for (count, p) in song_links.into_iter().enumerate() {
            if count % index == 0 && count != 0 {
                let p_element = get_tag(&download_page, "p[align='left']");
                let b_element = get_tag(&p_element.join(""), "b");
                let song_name = get_text(&b_element[2], "b");
                let save_file = target_dir.to_owned() + &album_dir + &song_name + file_extension;
                let save_file_path = Path::new(&save_file);
                if !save_file_path.exists() {
                    pool.execute(move || {
                        println!("\tDownloading {} ...", song_name);
                        let url = &p.url.unwrap();
                        let client = Client::builder().timeout(None).build().unwrap();
                        let resp = client.get(url).send().unwrap();
                        let audio = resp.bytes();
                        match audio {
                            Ok(bytes) => {
                                    let mut out = File::create(save_file).unwrap();
                                    io::copy(&mut bytes.as_ref(), &mut out)
                                        .expect("{song_name} failed to download");
                                    println!("\t{} downloaded succesfully!", song_name);
                            }
                            Err(err) => {
                                println!("{} download failed: {}", song_name, err);
                            }
                        }
                    });
                }
            }
        }
    }
    println!("Finished dowloading album.");
}
fn get_html(url: &str) -> String {
    let response = get(url);
    response.unwrap().text().unwrap()
}
fn get_element_links(html: &str, element: &str) -> Vec<Links> {
    let document = Html::parse_document(html);
    let html_selector = Selector::parse(element).unwrap();
    let html_elements = document.select(&html_selector);
    let mut elements: Vec<Links> = Vec::new();
    for element in html_elements {
        let url = element
            .select(&Selector::parse("a").unwrap())
            .next()
            .and_then(|a| a.value().attr("href"))
            .map(str::to_owned);
        let download_button = Links { url };
        elements.push(download_button);
    }
    elements
}
fn get_tag(text: &str, tag: &str) -> Vec<String> {
    let html = Html::parse_document(text);
    let tag_selector = Selector::parse(tag).unwrap();
    let mut result: Vec<String> = vec![];
    for element in html.select(&tag_selector) {
        result.push(element.html());
    }
    result
}
fn get_text(text: &str, tag: &str) -> String {
    let light_cone_html = Html::parse_fragment(text);
    let selector = Selector::parse(tag).unwrap();
    let result = light_cone_html.select(&selector).next().unwrap();
    result.text().collect()
}
fn scrub_windows_dir_name(dir_str: &str) -> String {
    let pattern = Regex::new("[<>:\"/\\|?*.]").unwrap();
    pattern.replace_all(dir_str, "").to_string()
}
