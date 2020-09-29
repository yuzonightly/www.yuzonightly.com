#![feature(proc_macro_hygiene, decl_macro)]
#![cfg_attr(test, deny(warnings))]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate rocket;
extern crate serde_json;
#[macro_use]
extern crate serde;
extern crate fluent_bundle;
extern crate rocket_contrib;
extern crate sass_rs;
extern crate toml;

mod caching;

use caching::{Cached, Caching};
use rocket::http::hyper::header::CacheDirective;
use rocket::response::NamedFile;
use rocket_contrib::templates::Template;
use sass_rs::{compile_file, Options};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::Hasher;
use std::path::{Path, PathBuf};

// I like when things starts making sense to me. 

lazy_static! {
    static ref ASSETS: AssetFiles = {
        let app_css_file = compile_sass("app");
        // let fonts_css_file = compile_sass("fonts");
        // let vendor_css_file = concat_vendor_css(vec!["tachyons"]);
        // let app_js_file = concat_app_js(vec!["tools-install"]);
        let fonts_css_file = "".into();
        let vendor_css_file = "".into();
        let app_js_file = "".into();
        AssetFiles {
            css: CSSFiles {
                app: app_css_file,
                fonts: fonts_css_file,
                vendor: vendor_css_file,
            },
            js: JSFiles {app:app_js_file},
        }
    };
}

#[derive(Serialize)]
struct Context<T: ::serde::Serialize> {
    page: String,
    title: String,
    parent: &'static str,
    is_landing: bool,
    data: T,
    baseurl: String,
    assets: &'static AssetFiles,
    pagename: String,
}

impl<T: ::serde::Serialize> Context<T> {
    fn new(page: String, title: &str, is_landing: bool, data: T, pagename: String) -> Self {
        Self {
            page,
            title: title.into(),
            parent: LAYOUT,
            is_landing,
            data,
            baseurl: baseurl(),
            assets: &ASSETS,
            pagename,
        }
    }
}

#[derive(Clone, Serialize)]
struct CSSFiles {
    app: String,
    fonts: String,
    vendor: String,
}

#[derive(Clone, Serialize)]
struct JSFiles {
    app: String,
}

#[derive(Clone, Serialize)]
struct AssetFiles {
    css: CSSFiles,
    js: JSFiles,
}

static LAYOUT: &str = "components/layout";

fn baseurl() -> String {
    String::new()
}

#[get("/components/<_file..>", rank = 1)]
fn components(_file: PathBuf) -> Template {
    not_found()
}

#[get("/static/<file..>", rank = 1)]
fn files(file: PathBuf) -> Option<Cached<NamedFile>> {
    NamedFile::open(Path::new("static/").join(file))
        .ok()
        .map(|file| file.cached(vec![CacheDirective::MaxAge(3600)]))
}

#[get("/projects")]
fn projects() -> Template {
    render_projects()
}

#[get("/writings")]
fn writings() -> Template {
    render_writings()
}

#[get("/")]
fn index() -> Template {
    render_index()
}

#[catch(404)]
fn not_found() -> Template {
    let page = "404";
    let context = Context::new("404".into(), "404!", false, (), "YuzoNightly".into());

    Template::render(page, &context)
}

#[catch(500)]
fn catch_error() -> Template {
    not_found()
}

fn hash_css(css: &str) -> String {
    let mut hasher = DefaultHasher::new();
    hasher.write(css.as_bytes());

    hasher.finish().to_string()
}

fn compile_sass(filename: &str) -> String {
    let scss_file = format!("./src/styles/{}.scss", filename);
    let css = compile_file(&scss_file, Options::default())
        .unwrap_or_else(|_| panic!("Could not compile sass: {}", &scss_file));
    let css_sha = format!("{}_{}", filename, hash_css(&css));
    let css_file = format!("./static/styles/{}.css", css_sha);
    fs::write(&css_file, css.into_bytes())
        .unwrap_or_else(|_| panic!("Could not write css file: {}", &css_file));

    // ./static/styles/{}.css
    String::from(&css_file[1..])
}

#[allow(dead_code)]
fn concat_vendor_css(files: Vec<&str>) -> String {
    let mut concat = String::new();
    for file in files {
        let vendor_path = format!("./static/styles/{}.css", file);
        let contents = fs::read_to_string(vendor_path).expect("Could not read vendor css");
        concat.push_str(&contents);
    }

    let css_sha = format!("vendor_{}", hash_css(&concat));
    let css_path = format!("./static/styles/{}.css", &css_sha);

    fs::write(&css_path, &concat).expect("Could not write vendor css");

    String::from(&css_path[1..])
}

#[allow(dead_code)]
fn concat_app_js(files: Vec<&str>) -> String {
    let mut concat = String::new();
    for file in files {
        let vendor_path = format!("./static/scripts/{}.js", file);
        let contents = fs::read_to_string(vendor_path).expect("Could not read app js");
        concat.push_str(&contents);
    }

    let js_sha = format!("app_{}", hash_css(&concat));
    let js_path = format!("./static/scripts/{}.js", &js_sha);

    fs::write(&js_path, &concat).expect("Could not write app js");

    String::from(&js_path[1..])
}

fn render_index() -> Template {
    let page = "index".to_string();
    let context = Context::new(page.clone(), "", true, (), "YuzoNightly".into());

    Template::render(page, &context)
}

fn render_projects() -> Template {
    let page = "projects/index".to_string();
    let context = Context::new(page.clone(), "Projects", true, (), "YuzoNightly".into());

    Template::render(page, &context)
}

fn render_writings() -> Template {
    let page = "writings/index".to_string();
    let context = Context::new(page.clone(), "Writings", true, (), "YuzoNightly".into());

    Template::render(page, &context)
}

fn main() {
    rocket::ignite()
        .attach(Template::fairing())
        .mount("/", routes![index, files, projects, writings])
        .register(catchers![not_found, catch_error])
        .launch();
}
