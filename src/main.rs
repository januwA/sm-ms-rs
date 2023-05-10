#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use eframe::egui::output::OpenUrl;
use eframe::egui::Ui;
use eframe::{
    egui::{self, RichText},
    epaint::Color32,
};
use egui_extras::RetainedImage;
use poll_promise::Promise;
use tokio::runtime::Runtime;

mod api;
mod cache;
mod util;

const K_IMAGE_MAX_WIDTH: f32 = 200.0;

fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    let cache_data = cache::SmMsCacheData::get_or_create();

    let mut options = eframe::NativeOptions::default();

    // options.initial_window_pos = Some([0f32, 0f32].into());
    options.min_window_size = Some([600f32, 400f32].into());

    eframe::run_native(
        "sm ms",
        options,
        Box::new(|cc| Box::new(MyApp::new(cc, cache_data))),
    )
}

fn setup_custom_fonts(ctx: &egui::Context) {
    // 从默认字体开始（我们将添加而不是替换它们）
    let mut fonts = egui::FontDefinitions::default();

    // 加载系统字体
    let font = std::fs::read("c:/Windows/Fonts/msyh.ttc").unwrap();
    fonts
        .font_data
        .insert("my_font".to_owned(), egui::FontData::from_owned(font));

    // 安装我的字体
    // fonts.font_data.insert(
    //     "my_font".to_owned(),
    //     egui::FontData::from_owned(include_bytes!(
    //         "../font/YeZiGongChangChuanQiuShaXingKai-2.ttf"
    //     )),
    // );

    // 对于比例文本，将我的字体放在第一位（最高优先级）
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "my_font".to_owned());

    // Put my font as last fallback for monospace:
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("my_font".to_owned());

    // 告诉 egui 使用这些字体
    ctx.set_fonts(fonts);
}

/* #region UploadHistoryDataUi */
struct UploadHistoryDataUi {
    data: api::UploadHistoryData,
    image_p: Promise<Result<RetainedImage, String>>,
}

impl UploadHistoryDataUi {
    fn from_data(data: api::UploadHistoryData, ctx: egui::Context) -> Self {
        let (sender, image_p) = Promise::new();
        let request = ehttp::Request::get(&data.url);
        ehttp::fetch(request, move |response| {
            let image = response.and_then(parse_ehttp_response);
            sender.send(image);
            ctx.request_repaint();
        });

        UploadHistoryDataUi { data, image_p }
    }
}
/* #endregion */

struct MyApp {
    logout_model_open: bool,

    delete_image_model_open: bool,
    delete_img_hash: Option<String>,

    /* #region login */
    username: String,
    password: String,
    login_loading: bool,
    login_err_o_s: Option<String>,
    token: String,
    token_o_p: Option<Promise<anyhow::Result<String>>>,
    /* #endregion */


    /* #region tab */
    tab: Vec<String>,
    tab_index: usize,
    /* #endregion */

    /* #region profile */
    profile_o_p: Option<Promise<anyhow::Result<api::ProfileData>>>,
    /* #endregion */

    /* #region upload history */
    upload_history_o_p: Option<Promise<anyhow::Result<Vec<UploadHistoryDataUi>>>>,
    /* #endregion */
    rt: Runtime,
}

/* #region MyApp constructor */
impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>, cache_data: Option<cache::SmMsCacheData>) -> Self {
        setup_custom_fonts(&cc.egui_ctx);

        let mut my = Self {
            logout_model_open: false,
            upload_history_o_p: None,
            profile_o_p: None,
            tab: vec![
                String::from("Upload History"),
                String::from("Now Upload"),
                String::from("Profile"),
            ],
            tab_index: 0,
            username: "ajanuw1995@gmail.com".to_string(),
            password: "".to_string(),
            login_err_o_s: None,
            login_loading: false,
            token: "".to_string(),
            token_o_p: None,
            rt: tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap(),
            delete_image_model_open: false,
            delete_img_hash: None,
        };

        if let Some(cache_data) = cache_data {
            // 从缓存中初始化token
            if let Some(token) = cache_data.token {
                my.token = token.clone();
                let (s, p) = Promise::new();
                my.token_o_p = Some(p);
                s.send(Ok(token.clone()));
            }
        }

        my.init();
        my
    }

    fn init(&mut self) {}
}
/* #endregion */

/* #region MyApp methods */
impl MyApp {
    fn get_profile_data(&mut self, ctx: &egui::Context) {
        // self.profile_o_p = None;
        let token = self.token.clone();
        self.profile_o_p.get_or_insert_with(|| {
            let (sender, promise) = Promise::new();
            let ctx = ctx.clone();
            self.rt.spawn(async move {
                let res_result = api::profile(token).await;
                sender.send(res_result);
                ctx.request_repaint();
            });
            promise
        });
    }

    fn get_upload_history_data(&mut self, ctx: &egui::Context) {
        // self.upload_history_o_p = None;

        let token = self.token.clone();

        self.upload_history_o_p.get_or_insert_with(|| {
            let (sender, promise) = Promise::new();
            let ctx = ctx.clone();
            self.rt.spawn(async move {
                let res_result = api::upload_history(token, 0).await;

                // Vec<api::UploadHistoryData> to Vec<api::UploadHistoryDataUi>
                let res_result_ui = res_result.and_then(|o: Vec<api::UploadHistoryData>| {
                    Ok(o.into_iter()
                        .map(|upload_history_data| {
                            UploadHistoryDataUi::from_data(upload_history_data, ctx.clone())
                        })
                        .collect())
                });

                sender.send(res_result_ui);
            });
            promise
        });
    }

    fn tab_item_click(&mut self, idx: usize, ctx: &egui::Context) {
        self.tab_index = idx;
        match self.tab_index {
            0 => self.get_upload_history_data(ctx),
            1 => {}
            2 => self.get_profile_data(ctx),

            _ => todo!(),
        }
    }
}
/* #endregion */

/* #region MyApp widgets */
impl MyApp {
    /// 登录界面
    fn widget_login(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("用户名: ").size(20.0));
                    ui.text_edit_singleline(&mut self.username);
                });
                ui.horizontal(|ui| {
                    ui.label(RichText::new("密码: ").size(20.0));
                    ui.add(egui::TextEdit::singleline(&mut self.password).password(true));
                });
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(
                            !self.login_loading,
                            egui::Button::new(RichText::new("登录").size(20.0)),
                        )
                        .clicked()
                    {
                        self.token_o_p = None;
                        self.login_err_o_s = None;
                        let (u, p) = (self.username.clone(), self.password.clone());
                        self.token_o_p.get_or_insert_with(|| {
                            let (sender, promise) = Promise::new();
                            self.rt.spawn(async move {
                                let res_result = api::token(u, p).await;
                                sender.send(res_result);
                            });
                            promise
                        });
                    }

                    if self.login_loading {
                        ui.spinner();
                    }
                });

                if let Some(login_err) = self.login_err_o_s.as_mut() {
                    egui::TextEdit::multiline(login_err)
                        .text_color(Color32::RED)
                        .show(ui);
                }
            });
        });
    }

    fn widget_tags(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        ui.horizontal(|ui| {
            self.tab.clone().iter().enumerate().for_each(|(i, label)| {
                if ui.selectable_label(self.tab_index == i, label).clicked() {
                    self.tab_item_click(i, ctx);
                }
            });
        });
    }

    // 显示上传的历史图片
    fn widget_images_list(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        let Some(upload_history_p) = &self.upload_history_o_p else {
            return;
         };
        match upload_history_p.ready() {
            Some(result) => match result {
                Ok(upload_history_v) => {
                    egui::ScrollArea::vertical()
                        .always_show_scroll(true)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.horizontal_wrapped(|ui| {
                                egui::Grid::new("some_unique_id").show(ui, |ui| {
                                    for (i, data) in upload_history_v.iter().enumerate() {
                                        let item = (ctx.screen_rect().width() / K_IMAGE_MAX_WIDTH)
                                            .floor()
                                            as usize;

                                        if i % item == 0 {
                                            ui.end_row();
                                        } else {
                                            ui.vertical(|ui| {
                                                if let Some(Ok(image)) = data.image_p.ready() {
                                                    image.show_max_size(
                                                        ui,
                                                        [K_IMAGE_MAX_WIDTH, K_IMAGE_MAX_WIDTH]
                                                            .into(),
                                                    );
                                                } else {
                                                    ui.spinner();
                                                }

                                                ui.horizontal(|ui| {
                                                    if ui.button("复制 url").clicked() {
                                                        ui.output_mut(|o| {
                                                            o.copied_text = data.data.url.clone()
                                                        });
                                                    }
                                                    if ui.button("打开 url").clicked() {
                                                        ui.output_mut(|o| {
                                                            o.open_url = Some(OpenUrl {
                                                                url: data.data.url.clone(),
                                                                new_tab: true,
                                                            });
                                                        });
                                                    }

                                                    if ui.button("删除").clicked() {
                                                        self.delete_img_hash =
                                                            Some(data.data.hash.clone());
                                                        self.delete_image_model_open = true;
                                                    }
                                                });
                                            });
                                        }
                                    }
                                });

                                // for data in upload_history_v {
                                // ui.with_layout(
                                //     egui::Layout::top_down(egui::Align::TOP),
                                //     |ui| {
                                //         if let Some(Ok(image)) = data.image_p.ready() {
                                //             image.show_max_size(
                                //                 ui,
                                //                 egui::Vec2::new(300.0, 300.0),
                                //             );
                                //         } else {
                                //             ui.spinner();
                                //         }
                                //         if ui.button("复制url").clicked() {
                                //             ui.output_mut(|o| {
                                //                 o.copied_text = data.data.url.clone()
                                //             });
                                //         }
                                //     },
                                // );
                                // }
                            });
                        });
                }
                Err(err) => {
                    ui.label(
                        RichText::new(&err.to_string())
                            .size(20.0)
                            .color(Color32::RED),
                    );
                }
            },
            _ => {
                ui.spinner();
            }
        }
    }

    // 显示账号信息
    fn widget_profile(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        if let Some(profile_p) = &self.profile_o_p {
            match profile_p.ready() {
                Some(result) => match result {
                    Ok(profile_data) => {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("username: ").size(20.0));
                                ui.label(RichText::new(&profile_data.username).size(20.0));
                            });
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("email: ").size(20.0));
                                ui.label(RichText::new(&profile_data.email).size(20.0));
                            });
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("role: ").size(20.0));
                                ui.label(RichText::new(&profile_data.role).size(20.0));
                            });
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("disk_usage: ").size(20.0));
                                ui.label(RichText::new(&profile_data.disk_usage).size(20.0));
                            });
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("disk_limit: ").size(20.0));
                                ui.label(RichText::new(&profile_data.disk_limit).size(20.0));
                            });
                        });
                    }
                    Err(err) => {
                        ui.label(
                            RichText::new(&err.to_string())
                                .size(20.0)
                                .color(Color32::RED),
                        );
                    }
                },
                _ => {
                    ui.spinner();
                }
            }
        };

        if ui
            .button(RichText::new("退出登录").color(Color32::GREEN))
            .clicked()
        {
            self.logout_model_open = true;
        };
    }

    /// 登录后界面
    fn widget_dashboard(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                // tabs
                self.widget_tags(ui, ctx);

                match self.tab_index {
                    0 => self.widget_images_list(ui, ctx),
                    1 => {
                        ui.label(&self.tab[self.tab_index]);
                    }
                    2 => self.widget_profile(ui, ctx),
                    _ => {
                        ui.label("??");
                    }
                }
            });
        });
    }
}
/* #endregion */

/* #region MyApp update */
impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.logout_model_open {
            egui::Window::new("Modal Window")
                .default_open(true)
                .default_width(120f32)
                .default_height(80f32)
                .show(ctx, |ui| {
                    ui.vertical(|ui| {
                        ui.label("确定退出登陆吗?");

                        ui.horizontal(|ui| {
                            if ui
                                .button(RichText::new("确定").color(Color32::BLUE))
                                .clicked()
                            {
                                self.token.clear();
                                self.token_o_p = None;
                                cache::SmMsCacheData::save(cache::SmMsCacheData { token: None });
                                self.logout_model_open = false;
                            }

                            if ui.button(RichText::new("取消")).clicked() {
                                self.logout_model_open = false;
                            }
                        });
                    });
                });
        }

        if self.delete_image_model_open {
            egui::Window::new("Modal Window")
                .default_open(true)
                .default_width(120f32)
                .default_height(80f32)
                .show(ctx, |ui| {
                    ui.vertical(|ui| {
                        ui.label("确定删除吗?");

                        ui.horizontal(|ui| {
                            if ui
                                .button(RichText::new("确定").color(Color32::BLUE))
                                .clicked()
                            {
                                self.delete_image_model_open = false;
                                let mut upload_history_o_p = self.upload_history_o_p.as_mut();
                                let token = self.token.clone();
                                let hash = self.delete_img_hash.clone().unwrap();
                                self.rt.spawn(async move {
                                    let res = api::delete_image(token, hash).await;
                                    if let Ok(r) = res {
                                        if r {
                                            upload_history_o_p = None;
                                            self.get_upload_history_data(&ctx);
                                        }
                                    }
                                });
                            }

                            if ui.button(RichText::new("取消")).clicked() {
                                self.delete_image_model_open = false;
                            }
                        });
                    });
                });
        }

        if self.token_o_p.is_none() {
            self.widget_login(ctx);
        } else {
            match self.token_o_p.as_mut().unwrap().ready() {
                Some(result) => match result {
                    Ok(token) => {
                        self.login_loading = false;
                        self.token = token.clone();

                        cache::SmMsCacheData::save(cache::SmMsCacheData {
                            token: Some(token.clone()),
                        })
                        .unwrap();

                        // 加载一下数据
                        self.tab_item_click(self.tab_index, ctx);
                        self.widget_dashboard(ctx);
                    }
                    Err(err) => {
                        self.login_loading = false;
                        self.login_err_o_s = Some(err.to_string());
                        self.token_o_p = None;
                        self.widget_login(ctx);
                    }
                },
                _ => {
                    self.login_loading = true;
                    self.widget_login(ctx);
                }
            }
        }
    }
}

/* #endregion */

#[allow(clippy::needless_pass_by_value)]
fn parse_ehttp_response(response: ehttp::Response) -> Result<RetainedImage, String> {
    let content_type = response.content_type().unwrap_or_default();
    if content_type.starts_with("image/") {
        RetainedImage::from_image_bytes(&response.url, &response.bytes)
    } else {
        Err(format!(
            "Expected image, found content-type {:?}",
            content_type
        ))
    }
}
