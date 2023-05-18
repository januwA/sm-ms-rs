#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use eframe::egui::output::OpenUrl;
use eframe::egui::{Hyperlink, Ui};
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
mod widget;

const K_IMAGE_MAX_WIDTH: f32 = 200.0;

fn main() -> Result<(), eframe::Error> {
    env_logger::init();
    let mut options = eframe::NativeOptions::default();
    options.icon_data = Some(util::load_app_icon());

    // options.initial_window_pos = Some([0f32, 0f32].into());
    options.min_window_size = Some([600f32, 400f32].into());

    eframe::run_native(
        "sm ms",
        options,
        Box::new(|cc| Box::new(SmMsApp::new(cc, cache::SmMsCacheData::get_or_create()))),
    )
}

/* #region UploadHistoryDataUi */
struct UploadHistoryDataUi {
    data: api::UploadHistoryData,
    image_p: Promise<Result<RetainedImage, String>>,
}

impl UploadHistoryDataUi {
    fn from_data(data: api::UploadHistoryData, ctx: egui::Context) -> Self {
        let (sender, image_p) = Promise::new();
        tokio::spawn(async {
            todo!();
        });
        let request = ehttp::Request::get(&data.url);
        ehttp::fetch(request, move |response| {
            let image = response.and_then(util::parse_ehttp_response);
            sender.send(image);
            ctx.request_repaint();
        });

        UploadHistoryDataUi { data, image_p }
    }
}
/* #endregion */

struct SmMsApp {
    upload_path: String,
    uplaod_res_msg: String,

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

impl Default for SmMsApp {
    fn default() -> Self {
        Self {
            upload_path: Default::default(),
            uplaod_res_msg: Default::default(),
            delete_image_model_open: Default::default(),
            delete_img_hash: Default::default(),
            username: Default::default(),
            password: Default::default(),
            login_loading: Default::default(),
            login_err_o_s: Default::default(),
            token: Default::default(),
            token_o_p: Default::default(),
            tab:vec![
                String::from("Upload History"),
                String::from("Now Upload"),
                String::from("Profile"),
            ],
            tab_index: Default::default(),
            profile_o_p: Default::default(),
            upload_history_o_p: Default::default(),
            rt: tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap(),
        }
    }
}

/* #region MyApp constructor */
impl SmMsApp {
    fn new(cc: &eframe::CreationContext<'_>, cache_data: Option<cache::SmMsCacheData>) -> Self {
        util::setup_custom_fonts(&cc.egui_ctx);
        let mut my = Self::default();
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
impl SmMsApp {
    fn upload(&mut self) {
        self.uplaod_res_msg.clear();

        if self.upload_path.is_empty() {
            self.uplaod_res_msg = "请填写上传本地文件路径".to_string();
            return;
        }

        if !std::path::Path::new(&self.upload_path).exists() {
            self.uplaod_res_msg = "文件不存在".to_string();
            return;
        }

        self.uplaod_res_msg = "上传中...".to_string();

        let res = self
            .rt
            .block_on(async { api::upload(&self.token, &self.upload_path).await });

        match res {
            Ok(_) => {
                self.uplaod_res_msg = "上传成功".to_string();
                self.upload_history_o_p = None;
            }
            Err(err) => self.uplaod_res_msg = err.to_string(),
        };
    }

    fn get_profile_data(&mut self, ctx: &egui::Context) {
        self.profile_o_p.get_or_insert_with(|| {
            let (sender, promise) = Promise::new();
            let token = self.token.clone();
            let ctx = ctx.clone();

            self.rt.spawn(async move {
                let res_result = api::profile(&token).await;
                sender.send(res_result);
                ctx.request_repaint();
            });
            promise
        });
    }

    fn get_upload_history_data(&mut self, ctx: &egui::Context) {
        self.upload_history_o_p.get_or_insert_with(|| {
            let (sender, promise) = Promise::new();
            let ctx = ctx.clone();
            let token = self.token.clone();
            self.rt.spawn(async move {
                let res_result = api::upload_history(&token).await;

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

/* #region MyApp panel */
impl SmMsApp {
    /// 登录界面
    fn login_panel(&mut self, ctx: &egui::Context) {
        let _my_frame = egui::containers::Frame {
            inner_margin: egui::style::Margin {
                left: 10.,
                right: 10.,
                top: 10.,
                bottom: 10.,
            },
            outer_margin: egui::style::Margin {
                left: 10.,
                right: 10.,
                top: 10.,
                bottom: 10.,
            },
            rounding: egui::Rounding {
                nw: 1.0,
                ne: 1.0,
                sw: 1.0,
                se: 1.0,
            },
            shadow: eframe::epaint::Shadow {
                extrusion: 1.0,
                color: Color32::YELLOW,
            },
            fill: Color32::LIGHT_BLUE,
            stroke: egui::Stroke::new(2.0, Color32::GOLD),
        };

        egui::CentralPanel::default()
            // .frame(my_frame)
            .show(ctx, |ui| {
                egui::Grid::new("login")
                    .num_columns(2)
                    .striped(false)
                    .show(ui, |ui| {
                        ui.label("用户名:");
                        ui.add(egui::TextEdit::singleline(&mut self.username));
                        ui.end_row();

                        ui.label("密码:");
                        ui.add(egui::TextEdit::singleline(&mut self.password).password(true));
                        ui.end_row();
                    });

                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(!self.login_loading, egui::Button::new("登录"))
                        .clicked()
                        && !self.username.is_empty()
                        && !self.password.is_empty()
                    {
                        self.token_o_p = None;
                        self.login_err_o_s = None;
                        self.token_o_p.get_or_insert_with(|| {
                            let (u, p) = (self.username.clone(), self.password.clone());
                            let (sender, promise) = Promise::new();
                            self.rt.spawn(async move {
                                let res_result = api::token(&u, &p).await;
                                sender.send(res_result);
                            });
                            promise
                        });
                    }

                    if self.login_loading {
                        ui.spinner();
                    }
                });

                ui.add(Hyperlink::from_label_and_url(
                    "Register",
                    "https://sm.ms/register",
                ));

                if let Some(login_err) = self.login_err_o_s.as_mut() {
                    egui::TextEdit::multiline(login_err)
                        .text_color(Color32::RED)
                        .show(ui);
                }
            });
    }

    fn tabs_panel(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        ui.horizontal(|ui| {
            self.tab.clone().iter().enumerate().for_each(|(i, label)| {
                if ui.selectable_label(self.tab_index == i, label).clicked() {
                    self.tab_item_click(i, ctx);
                }
            });
        });
    }

    // 显示上传的历史图片
    fn images_grid_panel(&mut self, ui: &mut Ui, ctx: &egui::Context) {
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
                            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                                egui::Grid::new("images grid").show(ui, |ui| {
                                    for (i, data) in upload_history_v.iter().enumerate() {
                                        let item = (ctx.screen_rect().width() / K_IMAGE_MAX_WIDTH)
                                            .floor()
                                            as usize;

                                        if i % item == 0 {
                                            ui.end_row();
                                        } else {
                                            ui.vertical(|ui| {
                                                ui.with_layout(
                                                    egui::Layout::top_down(egui::Align::Center),
                                                    |ui| {
                                                        if let Some(Ok(image)) =
                                                            data.image_p.ready()
                                                        {
                                                            image.show_max_size(
                                                                ui,
                                                                [
                                                                    K_IMAGE_MAX_WIDTH,
                                                                    K_IMAGE_MAX_WIDTH,
                                                                ]
                                                                .into(),
                                                            );
                                                        } else {
                                                            ui.spinner();
                                                        }
                                                    },
                                                );

                                                ui.with_layout(
                                                    egui::Layout::bottom_up(egui::Align::LEFT),
                                                    |ui| {
                                                        ui.horizontal(|ui| {
                                                            if ui.button("复制 url").clicked() {
                                                                ui.output_mut(|o| {
                                                                    o.copied_text =
                                                                        data.data.url.clone()
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
                                                    },
                                                );
                                            });
                                        }
                                    }
                                });
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
    fn profile_panel(&mut self, ui: &mut Ui, _ctx: &egui::Context) {
        if let Some(profile_p) = &self.profile_o_p {
            match profile_p.ready() {
                Some(result) => match result {
                    Ok(profile_data) => {
                        ui.vertical(|ui| {
                            widget::info_row(ui, "username: ", &profile_data.username);
                            widget::info_row(ui, "email: ", &profile_data.email);
                            widget::info_row(ui, "role: ", &profile_data.role);
                            widget::info_row(ui, "disk_usage: ", &profile_data.disk_usage);
                            widget::info_row(ui, "disk_limit: ", &profile_data.disk_limit);
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

        ui.separator();

        if widget::error_button(ui, "退出登录").clicked() {
            self.token.clear();
            self.token_o_p = None;
            cache::SmMsCacheData::save(cache::SmMsCacheData { token: None }).unwrap();
        };
    }

    /// 登录后界面
    fn dashboard_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                self.tabs_panel(ui, ctx);
                ui.separator();

                match self.tab_index {
                    0 => self.images_grid_panel(ui, ctx),
                    1 => {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label("文件路径：");
                                ui.text_edit_singleline(&mut self.upload_path);
                                if ui.button("上传").clicked() {
                                    self.upload();
                                }
                            });

                            ui.label(RichText::new(&self.uplaod_res_msg).color(Color32::RED));
                        });
                    }
                    2 => self.profile_panel(ui, ctx),
                    _ => {
                        ui.label("??");
                    }
                }
            });
        });
    }

    fn menu_panel(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // 顶部菜单栏
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.close();
                    }
                });
            });
        });
    }
}
/* #endregion */

impl eframe::App for SmMsApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.menu_panel(ctx, frame);

        if self.delete_image_model_open {
            egui::Window::new("Modal Window")
                .default_open(true)
                .default_width(120f32)
                .default_height(80f32)
                .show(ctx, |ui| {
                    ui.vertical(|ui| {
                        ui.label("确定删除吗?");

                        ui.horizontal(|ui| {
                            if widget::error_button(ui, "确定").clicked() {
                                let hash = self.delete_img_hash.clone().unwrap();
                                let res = self.rt.block_on(async {
                                    api::delete_image(&self.token, &hash).await
                                });
                                if res.is_ok() {
                                    self.upload_history_o_p = None;
                                    self.get_upload_history_data(&ctx);
                                }
                                self.delete_image_model_open = false;
                            }

                            if ui.button("取消").clicked() {
                                self.delete_image_model_open = false;
                            }
                        });
                    });
                });
        }

        if self.token_o_p.is_none() {
            self.login_panel(ctx);
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
                        self.dashboard_panel(ctx);
                    }
                    Err(err) => {
                        self.login_loading = false;
                        self.login_err_o_s = Some(err.to_string());
                        self.token_o_p = None;
                        self.login_panel(ctx);
                    }
                },
                _ => {
                    self.login_loading = true;
                    self.login_panel(ctx);
                }
            }
        }
    }
}
