/*
 * Copyright (c) 2020 Boucher, Antoni <bouanto@zoho.com>
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of
 * this software and associated documentation files (the "Software"), to deal in
 * the Software without restriction, including without limitation the rights to
 * use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
 * the Software, and to permit persons to whom the Software is furnished to do so,
 * subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
 * FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
 * COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
 * IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
 * CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 */

use gtk::{prelude::BuilderExtManual, Builder, Button, ButtonExt, Entry, EntryExt, Inhibit, Label, LabelExt, WidgetExt, Window, Stack, StackExt, GtkWindowExt, Box, ContainerExt, Expander, ExpanderExt, ExpanderBuilder, Image, ImageExt, Align, Separator};
use relm_derive::Msg;
use relm::{connect, Relm, Update, Widget, WidgetTest, Channel};
use webbrowser;
use egs_api::EpicGames;
use tokio::runtime::Runtime;
use gtk::Orientation::{Vertical, Horizontal};
use std::collections::HashMap;
use egs_api::api::types::{EpicAsset, AssetInfo, DownloadManifest};
use gdk_pixbuf::PixbufLoaderExt;
use std::thread;
use crate::Msg::{LoginOk, ProcessAssetList, ProcessAssetInfo, ProcessImage, ProcessDownloadManifest};
use egs_api::api::UserData;
use threadpool::ThreadPool;


struct Model {
    relm: Relm<Win>,
    assets: HashMap<String, EpicAsset>,
    download_manifests: HashMap<String, DownloadManifest>,
}

#[derive(Msg)]
enum Msg {
    GetSid,
    Login,
    LoginOk(UserData),
    ProcessAssetList(HashMap<String, HashMap<String, EpicAsset>>),
    ProcessAssetInfo(AssetInfo),
    ProcessImage((String, Vec<u8>)),
    LoadDownloadManifest(String),
    ProcessDownloadManifest(String, DownloadManifest),
    Quit,
}

// Create the structure that holds the widgets used in the view.
#[derive(Clone)]
struct Widgets {
    window: Window,
    get_sid_button: Button,
    login_button: Button,
    sid: Entry,
    stack: Stack,
    login_name: Label,
    assets_main_box: Box,
    asset_namespaces: HashMap<String, Expander>,
    asset_boxes: HashMap<String, Box>,
    asset_thumbnails: HashMap<String, Image>,
    asset_files: HashMap<String, Box>,
}

struct Win {
    model: Model,
    widgets: Widgets,
    epic_games: EpicGames,
}

impl Update for Win {
    // Specify the model used for this widget.
    type Model = Model;
    // Specify the model parameter used to init the model.
    type ModelParam = ();
    // Specify the type of the messages sent to the update function.
    type Msg = Msg;

    fn model(relm: &Relm<Self>, _: ()) -> Model {
        Model {
            relm: relm.clone(),
            assets: HashMap::new(),
            download_manifests: HashMap::new(),
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::Quit => gtk::main_quit(),
            Msg::GetSid => {
                webbrowser::open("https://www.epicgames.com/id/login?redirectUrl=https%3A%2F%2Fwww.epicgames.com%2Fid%2Fapi%2Fredirect").unwrap();
            }
            Msg::Login => {
                let sid = self.widgets.sid.get_text();
                let stream = self.model.relm.stream().clone();
                let (_channel, sender) = Channel::new(move |ud| {
                    match ud {
                        None => {}
                        Some(user_data) => { stream.emit(LoginOk(user_data)); }
                    }
                });

                let mut eg = self.epic_games.clone();
                thread::spawn(move || {
                    match Runtime::new().unwrap().block_on(eg.auth_sid(sid.as_str()))
                    {
                        None => {}
                        Some(exchange_token) => {
                            if Runtime::new().unwrap().block_on(eg.auth_code(exchange_token))
                            {
                                sender.send(Some(eg.user_details())).unwrap();
                            }
                        }
                    };
                });
            }
            Msg::LoginOk(user_data) => {
                &self.epic_games.set_user_details(user_data);
                &self.widgets.stack.set_visible_child_name("logged_in_overlay");
                &self.widgets.login_name.set_label(&self.epic_games.user_details().display_name.unwrap().as_str());

                let stream = self.model.relm.stream().clone();
                let (_channel, sender) = Channel::new(move |am| {
                    stream.emit(ProcessAssetList(am));
                });

                let mut eg = self.epic_games.clone();
                thread::spawn(move || {
                    let assets = Runtime::new().unwrap().block_on(eg.list_assets());
                    let mut asset_map: HashMap<String, HashMap<String, EpicAsset>> = HashMap::new();
                    for asset in assets {
                        match asset_map.get_mut(asset.namespace.as_str()) {
                            None => { asset_map.insert(asset.namespace.clone(), [(asset.catalog_item_id.clone(), asset)].iter().cloned().collect()); }
                            Some(existing) => { existing.insert(asset.catalog_item_id.clone(), asset); }
                        };
                    };
                    sender.send(asset_map).unwrap();
                });
            }
            ProcessAssetList(asset_map) => {
                for (namespace, filtered_assets) in asset_map {
                    println!("Adding namespace: {}", namespace);
                    let assets_box = Box::new(Vertical, 0);
                    for (id, a) in filtered_assets.clone() {
                        self.model.assets.insert(id, a.clone());
                        let asset_box = Box::new(Vertical, 0);

                        assets_box.add(&asset_box);
                        asset_box.add(&Separator::new(gtk::Orientation::Horizontal));
                        self.widgets.asset_boxes.insert(a.catalog_item_id.clone(), asset_box);
                    }

                    let stream = self.model.relm.stream().clone();
                    let (_channel, sender) = Channel::new(move |ai| {
                        stream.emit(ProcessAssetInfo(ai));
                    });

                    let eg = self.epic_games.clone();
                    let fa = filtered_assets.clone();
                    thread::spawn(move || {
                        let pool = ThreadPool::new(5);
                        for (_id, ass) in fa.clone() {
                            let mut e = eg.clone();
                            let s = sender.clone();
                            pool.execute(move || {
                                match Runtime::new().unwrap().block_on(e.get_asset_info(ass)) {
                                    None => {}
                                    Some(asset) => {
                                        s.send(asset).unwrap();
                                    }
                                };
                            });
                        }
                    });
                    assets_box.show_all();
                    let category = ExpanderBuilder::new().label(&format!("{} ({})", namespace, filtered_assets.keys().len())).child(&assets_box).build();
                    category.set_property_expand(true);
                    &self.widgets.assets_main_box.add(&gtk::Separator::new(Horizontal));
                    &self.widgets.assets_main_box.add(&category);
                    &self.widgets.asset_namespaces.insert(namespace, category);
                };
                &self.widgets.assets_main_box.show_all();
            }
            Msg::ProcessAssetInfo(asset) => {
                let asset_box = self.widgets.asset_boxes.get(asset.id.as_str()).unwrap();
                let name = Label::new(None);
                name.set_halign(Align::Start);
                name.set_markup(format!("<b>{}</b>", glib::markup_escape_text(&asset.title)).as_str());
                asset_box.add(&name);
                asset_box.set_margin_start(20);
                let details_box = Box::new(Horizontal, 5);
                let gtkimage = Image::new();
                gtkimage.set_margin_start(20);
                details_box.add(&gtkimage);
                let info_box = Box::new(Vertical, 0);
                let developer = Label::new(Some(format!("Developer: {}", &asset.developer).as_str()));
                developer.set_halign(Align::Start);
                info_box.add(&developer);
                let description = Label::new(None);
                description.set_halign(Align::Start);
                description.set_markup(format!("{}", glib::markup_escape_text(&asset.description)).as_str());
                description.set_property_wrap(true);
                info_box.set_property_expand(true);
                info_box.add(&description);
                details_box.add(&info_box);
                self.widgets.asset_thumbnails.insert(asset.id.clone(), gtkimage);

                for image in asset.key_images.clone() {
                    if image.type_field.eq_ignore_ascii_case("Thumbnail") {
                        println!("{}: {}", image.type_field, image.url);

                        let stream = self.model.relm.stream().clone();
                        let (_channel, sender) = Channel::new(move |(id, b)| {
                            stream.emit(ProcessImage((id, b)));
                        });

                        let id = asset.id.clone();
                        thread::spawn(move || {
                            match reqwest::blocking::get(&image.url) {
                                Ok(response) => {
                                    match response.bytes() {
                                        Ok(b) => {
                                            sender.send((id, Vec::from(b.as_ref()))).unwrap();
                                        }
                                        Err(_) => {}
                                    }
                                }
                                Err(_) => {}
                            };
                        });
                    }
                }
                let download_button: Button = Button::new();
                download_button.set_label("Download");
                details_box.add(&download_button);
                details_box.show_all();
                details_box.set_property_expand(true);
                asset_box.add(&details_box);
                let file_expander = ExpanderBuilder::new().label("File list").build();
                let file_box: Box = Box::new(Vertical, 0);
                file_expander.add(&file_box);

                self.widgets.asset_files.insert(asset.id.clone(), file_box);
                asset_box.add(&file_expander);
                asset_box.set_property_expand(true);
                asset_box.show_all();
                connect!(self.model.relm, file_expander,connect_property_expanded_notify(_), Msg::LoadDownloadManifest(asset.id.clone()));
            }
            Msg::ProcessImage((id, b)) => {
                match self.widgets.asset_thumbnails.get(&id) {
                    None => {}
                    Some(gtkimage) => {
                        let pixbuf_loader = gdk_pixbuf::PixbufLoader::new();
                        pixbuf_loader.write(&b).unwrap();
                        pixbuf_loader.close().ok();
                        gtkimage.set_from_pixbuf(pixbuf_loader.get_pixbuf().unwrap().scale_simple(60, 60, gdk_pixbuf::InterpType::Bilinear).as_ref());
                    }
                }
            }
            Msg::LoadDownloadManifest(id) => {
                let asset = match self.model.assets.get(id.as_str())
                {
                    None => { return; }
                    Some(a) => { a.clone() }
                };

                if let Some(_) = self.model.download_manifests.get(id.as_str()) { return; }
                let stream = self.model.relm.stream().clone();
                let (_channel, sender) = Channel::new(move |dm| {
                    stream.emit(ProcessDownloadManifest(id.clone(), dm));
                });

                let mut eg = self.epic_games.clone();
                thread::spawn(move || {
                    match Runtime::new().unwrap().block_on(eg.get_asset_manifest(asset)) {
                        None => {}
                        Some(manifest) => {
                            for elem in manifest.elements {
                                for man in elem.manifests {
                                    match Runtime::new().unwrap().block_on(eg.get_asset_download_manifest(man.clone())) {
                                        Ok(d) => {
                                            sender.send(d).unwrap();
                                            break;
                                        }
                                        Err(_) => {}
                                    };
                                }
                            }
                        }
                    };
                });
            }
            ProcessDownloadManifest(id, dm) => {
                self.model.download_manifests.insert(id.clone(), dm.clone());
                let file_list = match self.widgets.asset_files.get(id.as_str()) {
                    None => { return; }
                    Some(fl) => { fl }
                };
                for (file, _info) in dm.get_files() {
                    let file_box = Box::new(Horizontal, 0);
                    let label = Label::new(Some(&file));
                    label.set_halign(Align::Start);
                    label.set_ellipsize(pango::EllipsizeMode::Middle);
                    label.set_property_expand(true);
                    file_box.add(&label);
                    let download_button: Button = Button::new();
                    download_button.set_label("Download");
                    file_box.add(&download_button);
                    file_box.show_all();
                    file_list.add(&file_box);
                }
                file_list.show_all();
            }
        }
    }
}

impl Widget for Win {
    // Specify the type of the root widget.
    type Root = Window;

    // Return the root widget.
    fn root(&self) -> Self::Root {
        self.widgets.window.clone()
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        let glade_src = include_str!("gui.glade");
        let builder = Builder::from_string(glade_src);
        let window: Window = builder.get_object("window").unwrap();
        let get_sid_button: Button = builder.get_object("get_sid").unwrap();
        let login_button: Button = builder.get_object("login").unwrap();
        let sid_field: Entry = builder.get_object("sid").unwrap();
        let stack: Stack = builder.get_object("stack").unwrap();
        let login_name: Label = builder.get_object("login_name").unwrap();
        let assets_main_box: Box = builder.get_object("assets_main_box").unwrap();
        window.set_title("Epic Asset Browser");
        window.show_all();


        connect!(relm, get_sid_button, connect_clicked(_), Msg::GetSid);
        connect!(relm, login_button, connect_clicked(_), Msg::Login);
        connect!(relm, window, connect_delete_event(_, _), return (Some(Msg::Quit), Inhibit(false)));

        Win {
            model,
            widgets: Widgets {
                window,
                get_sid_button,
                login_button,
                sid: sid_field,
                stack,
                login_name,
                assets_main_box,
                asset_namespaces: Default::default(),
                asset_boxes: Default::default(),
                asset_thumbnails: Default::default(),
                asset_files: Default::default(),
            },
            epic_games: EpicGames::new(),
        }
    }
}

impl WidgetTest for Win {
    type Streams = ();

    fn get_streams(&self) -> Self::Streams {}

    type Widgets = Widgets;

    fn get_widgets(&self) -> Self::Widgets {
        self.widgets.clone()
    }
}

fn main() {
    Win::run(()).expect("Win::run failed");
}