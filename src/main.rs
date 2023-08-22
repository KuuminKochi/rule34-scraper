use std::{vec};
use std::process::Command;
use std::fs;
use tokio;
use clap::Parser;
use reqwest;
use scraper::{Html, Selector};

#[derive(Debug)]
struct Media {
    title: String,
    url: String,
    file_type: String,
}

struct MediaBuilder {
    title: String,
    url: String,
    file_type: String,
}

impl MediaBuilder {
    fn new() -> Self {
        MediaBuilder {
            title: "Error".to_string(),
            url: "https://i.pinimg.com/originals/13/92/6c/13926cfb3fd8818166d8b3149e0696de.jpg".to_string(),
            file_type: "jpg".to_string(),
        }
    }
    
    fn add_url(mut self, url: String) -> Self {
        self.url = url;
        self
    }
    
    fn add_title(mut self, title: String) -> Self {
        self.title = title;
        self
    }
    
    fn add_file_type(mut self, file_type: String) -> Self {
        self.file_type = file_type;
        self
    }

    fn build(self) -> Media {
        Media {
            title: self.title,
            url: self.url,
            file_type: self.file_type,
        }
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    url: String,

    /// Initial page to scrape
    #[arg(short, long, default_value_t = 0)]
    start_page: usize,

    /// Last page to scrape
    #[arg(short, long, default_value_t = 0)]
    last_page: usize,

    /// Output path
    #[arg(short, long, default_value = "./")]
    output_path: String,
}

async fn get_html(url: String) -> Result<String, reqwest::Error> {

    static APP_USER_AGENT: &str = concat!(
        env!("CARGO_PKG_NAME"),
        "/",
        env!("CARGO_PKG_VERSION"),
    );
    
    let client = reqwest::Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()?;
    
    let body = client.get(url)
        .send()
        .await?
        .text()
        .await?;
    
    Ok(body)
}

async fn html_parser(html: String) -> Html {
    let document = Html::parse_document(html.as_str());
    document
}

async fn source_list_extractor(document: Html) -> Vec<String> {
    let mut source_list: Vec<String> = vec![];
    let selector = Selector::parse(r#"a[style=""]"#).unwrap();
    
    for element in document.select(&selector) {
        let link = element.value().attr("href").unwrap().to_string();
        source_list.push(link);
    }

    source_list
}

async fn media_extractor(document: Html) -> Media {
    let image_selector = Selector::parse(r#"img[id="image"]"#).unwrap();
    let mp4_selector = Selector::parse(r#"source[type="video/mp4"]"#).unwrap();
    let mpeg_selector = Selector::parse(r#"source[type="video/mpeg"]"#).unwrap();
    let mpg_selector = Selector::parse(r#"source[type="video/mpg"]"#).unwrap();
    let webm_selector = Selector::parse(r#"source[type="video/webm"]"#).unwrap();
    let avi_selector = Selector::parse(r#"source[type="video/avi"]"#).unwrap();
    
    let selector_list: Vec<&Selector> = vec![&image_selector, &mp4_selector, &mpeg_selector, &mpg_selector, &webm_selector, &avi_selector];
    
    let mut link: String = "https://i.pinimg.com/originals/13/92/6c/13926cfb3fd8818166d8b3149e0696de.jpg".to_string();
    let mut title: String = "placeholder".to_string();
    let mut file_type: String = "jpeg".to_string();
    
    for selector in selector_list {
        
        let mut media_type = "image";
        let media_link = document.clone();
        
        if selector == &image_selector {
            media_type = "image";
        } else {
            media_type = "video"
        }
        
        if media_type == "image" {
            let media_link = media_link.select(&selector).next();
            
            if !(media_link == None) {
                link = media_link.unwrap().value().attr("src").unwrap().to_string();
                title = link.clone().replace("/", "-");
                file_type = get_format(&link).unwrap().to_string();
                break;
            }
        }

        if media_type == "video" {
            let media_link = media_link.select(&selector).next();
            
            if !(media_link == None) {
                link = media_link.unwrap().value().attr("src").unwrap().to_string();
                title = link.clone().replace("/", "-");
                file_type = get_format(&link).unwrap().to_string();
                break;
            }
        }
    }
    
    let media = MediaBuilder::new()
        .add_url(link)
        .add_title(title)
        .add_file_type(file_type)
        .build();
    
    media

}

fn get_format(url: &str) -> Option<&str> {
    if url.contains("jpeg") {
        Some(".jpeg")
    } else if url.contains("jpg") {
        Some(".jpg")
    } else if url.contains("png") {
        Some(".png")
    } else if url.contains("gif") {
        Some(".gif")
    } else if url.contains("webm") {
        Some(".webm")
    } else if url.contains("avi") {
        Some(".avi")
    } else if url.contains("mp4") {
        Some(".mp4")
    } else if url.contains("mpg") {
        Some(".mpg")
    } else if url.contains("mpeg") {
        Some(".mpeg")
    } else {
        None
    }
}

fn wget(media: Media, path: &String) {

    let file = format_title(&media.title) + &media.file_type;
    let file_path = path.to_owned() + &file;

    if fs::metadata(&file_path).is_ok() {
        println!("{} already exists", &file)       
        } else {
        let output = Command::new("wget")
            .args(&["-P", &path, "-O", &file_path, &media.url])
            .output()
            .expect("Wget Error");
        
        if output.status.success() {
            println!("{} downloaded successfully!", &file);
        } else {
            println!("FAILED FILES: {}", &file);
            println!("{:#?}", output);
        }
    }
    
}

fn next_page(url: &String, page: usize) -> String {
    let next_url: String = format!("{}&pid={}", url, &page * 42);
    return next_url;
}

fn format_title(title: &String) -> String {
    if title.len() > 60 {
       let new_title: String = (title.replace(" ", ""))[..60].to_string();
        new_title
    } else {
        title.replace(" ", "").to_string()
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    for html in args.start_page..=args.last_page {
        let url = next_page(&args.url, html);
        let html = get_html(url).await;
        let document = html_parser(html.unwrap()).await;
        let source_list = source_list_extractor(document).await;
        for source in source_list.iter() {
            let link = "https://rule34.xxx".to_string() + source;
            let source_html = get_html(link).await;
            let source_document = html_parser(source_html.unwrap()).await;
            let media = media_extractor(source_document).await;
            wget(media, &args.output_path);
        }
    }
}
