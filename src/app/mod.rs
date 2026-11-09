use std::path::PathBuf;

use eframe::egui::{self, Align, Color32, Layout, RichText, ScrollArea, TextEdit};
use rfd::FileDialog;

use crate::{
    db::{ColumnInfo, DatabaseManager, DbObject, QueryResult, TableData},
    ui::apply_theme,
};

const TABLE_PAGE_SIZE: usize = 200;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WorkspaceTab {
    Welcome,
    Data,
    Query,
    Structure,
    History,
}

pub struct OxideApp {
    database: DatabaseManager,
    objects: Vec<DbObject>,
    explorer_filter: String,
    selected_object: Option<DbObject>,
    selected_table: Option<String>,
    active_tab: WorkspaceTab,
    sql_editor: String,
    query_result: Option<QueryResult>,
    table_data: TableData,
    table_columns: Vec<ColumnInfo>,
    table_offset: usize,
    table_has_next: bool,
    query_history: Vec<String>,
    last_error: Option<String>,
    status_message: String,
    sqlite_version: Option<String>,
    show_about: bool,
}

impl OxideApp {
    pub fn new(creation_context: &eframe::CreationContext<'_>) -> Self {
        apply_theme(&creation_context.egui_ctx);
        Self {
            database: DatabaseManager::default(),
            objects: Vec::new(),
            explorer_filter: String::new(),
            selected_object: None,
            selected_table: None,
            active_tab: WorkspaceTab::Welcome,
            sql_editor: "SELECT name, type\nFROM sqlite_schema\nWHERE name NOT LIKE 'sqlite_%'\nORDER BY type, name;".to_owned(),
            query_result: None,
            table_data: TableData::default(),
            table_columns: Vec::new(),
            table_offset: 0,
            table_has_next: false,
            query_history: Vec::new(),
            last_error: None,
            status_message: "Ready · open a database to begin".to_owned(),
            sqlite_version: None,
            show_about: false,
        }
    }

    fn open_dialog(&mut self, read_only: bool) {
        let Some(path) = FileDialog::new()
            .add_filter("SQLite database", &["db", "sqlite", "sqlite3"])
            .add_filter("All files", &["*"])
            .pick_file()
        else {
            return;
        };
        self.open_database(path, read_only);
    }

    fn create_dialog(&mut self) {
        let Some(path) = FileDialog::new()
            .set_file_name("untitled.sqlite")
            .add_filter("SQLite database", &["sqlite", "db"])
            .save_file()
        else {
            return;
        };
        self.open_database(path, false);
    }

    fn open_database(&mut self, path: PathBuf, read_only: bool) {
        match self.database.open(&path, read_only) {
            Ok(()) => {
                self.last_error = None;
                self.status_message = format!(
                    "{} · {}",
                    if read_only { "Opened read-only" } else { "Connected" },
                    path.display()
                );
                self.selected_table = None;
                self.selected_object = None;
                self.table_offset = 0;
                self.refresh_database();
                self.active_tab = if self.selected_table.is_some() {
                    WorkspaceTab::Data
                } else {
                    WorkspaceTab::Query
                };
            }
            Err(error) => self.set_error(error),
        }
    }

    fn start_demo(&mut self) {
        match self.database.open_demo() {
            Ok(()) => {
                self.status_message = "Demo workspace connected · in-memory database".to_owned();
                self.last_error = None;
                self.selected_table = None;
                self.selected_object = None;
                self.table_offset = 0;
                self.refresh_database();
                self.select_named_object("customers");
            }
            Err(error) => self.set_error(error),
        }
    }

    fn refresh_database(&mut self) {
        match self.database.list_objects() {
            Ok(objects) => self.objects = objects,
            Err(error) => {
                self.set_error(error);
                return;
            }
        }
        self.sqlite_version = self.database.sqlite_version().ok();
        if self.selected_table.as_ref().is_some_and(|table| {
            !self.objects.iter().any(|object| {
                (object.kind == "table" || object.kind == "view") && &object.name == table
            })
        }) {
            self.selected_table = None;
            self.selected_object = None;
        }
        if self.selected_table.is_none()
            && let Some(object) = self.objects.iter().find(|object| object.kind == "table")
        {
            self.selected_table = Some(object.name.clone());
            self.selected_object = Some(object.clone());
        }
        if let Some(table) = self.selected_table.clone() {
            self.load_table(&table);
        }
    }

    fn select_named_object(&mut self, name: &str) {
        if let Some(object) = self.objects.iter().find(|object| object.name == name).cloned() {
            self.select_object(object);
        }
    }

    fn select_object(&mut self, object: DbObject) {
        self.last_error = None;
        self.selected_object = Some(object.clone());
        if object.kind == "table" || object.kind == "view" {
            self.selected_table = Some(object.name.clone());
            self.table_offset = 0;
            self.load_table(&object.name);
            self.active_tab = WorkspaceTab::Data;
        } else {
            self.selected_table = Some(object.table_name.clone());
            self.active_tab = WorkspaceTab::Structure;
            self.table_columns = self.database.table_columns(&object.table_name).unwrap_or_default();
        }
    }

    fn load_table(&mut self, table: &str) {
        match self.database.table_data(table, TABLE_PAGE_SIZE + 1, self.table_offset) {
            Ok(mut data) => {
                self.table_has_next = data.rows.len() > TABLE_PAGE_SIZE;
                data.rows.truncate(TABLE_PAGE_SIZE);
                self.table_data = data;
                self.table_columns = self.database.table_columns(table).unwrap_or_default();
                self.last_error = None;
            }
            Err(error) => self.set_error(error),
        }
    }

    fn run_query(&mut self) {
        let sql = self.sql_editor.trim().to_owned();
        if sql.is_empty() {
            return;
        }
        match self.database.execute(&sql) {
            Ok(result) => {
                self.query_result = Some(result);
                self.last_error = None;
                self.status_message = "SQL executed successfully".to_owned();
                self.query_history.push(sql);
                if self.query_history.len() > 100 {
                    self.query_history.remove(0);
                }
                self.refresh_database();
            }
            Err(error) => self.set_error(error),
        }
    }

    fn export_result(&mut self) {
        let Some(result) = self.query_result.clone() else {
            return;
        };
        if result.columns.is_empty() {
            self.status_message = "There is no tabular result to export".to_owned();
            return;
        }
        let Some(path) = FileDialog::new()
            .set_file_name("query-results.csv")
            .add_filter("CSV", &["csv"])
            .save_file()
        else {
            return;
        };
        match self.database.export_csv(&result, &path) {
            Ok(()) => self.status_message = format!("CSV exported · {}", path.display()),
            Err(error) => self.set_error(error),
        }
    }

    fn run_integrity_check(&mut self) {
        match self.database.integrity_check() {
            Ok(messages) => {
                let healthy = messages.iter().all(|message| message == "ok");
                self.status_message = if healthy {
                    "Integrity check passed · database is healthy".to_owned()
                } else {
                    format!("Integrity check: {}", messages.join("; "))
                };
                self.last_error = None;
            }
            Err(error) => self.set_error(error),
        }
    }

    fn set_error(&mut self, error: impl std::fmt::Display) {
        let message = error.to_string();
        // Keep diagnostics content-free: SQLite errors can echo user SQL or file paths.
        tracing::error!("application operation failed");
        self.last_error = Some(message.clone());
        self.status_message = "Operation failed".to_owned();
    }

    fn show_header(&mut self, root_ui: &mut egui::Ui) {
        egui::Panel::top("header")
            .frame(
                egui::Frame::new()
                    .fill(Color32::from_rgb(11, 17, 25))
                    .inner_margin(egui::Margin::symmetric(18, 10)),
            )
            .show(root_ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("◈ OXIDE")
                            .size(18.0)
                            .strong()
                            .color(Color32::from_rgb(113, 224, 204)),
                    );
                    ui.label(
                        RichText::new("STUDIO")
                            .size(11.0)
                            .strong()
                            .color(Color32::from_rgb(143, 161, 181)),
                    );
                    ui.separator();
                    if self.database.is_open() {
                        let name = self
                            .database
                            .path()
                            .and_then(|path| path.file_name())
                            .and_then(|name| name.to_str())
                            .unwrap_or("In-memory database");
                        ui.label(
                            RichText::new(format!("●  {name}"))
                                .color(Color32::from_rgb(113, 224, 204)),
                        );
                        if self.database.is_read_only() {
                            ui.label(
                                RichText::new("READ ONLY")
                                    .small()
                                    .color(Color32::from_rgb(239, 191, 103)),
                            );
                        }
                    } else {
                        ui.label(
                            RichText::new("No database connected")
                                .color(Color32::from_rgb(147, 163, 181)),
                        );
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button("About").clicked() {
                            self.show_about = true;
                        }
                        if ui.button("Run  Ctrl+Enter").clicked() {
                            self.active_tab = WorkspaceTab::Query;
                            self.run_query();
                        }
                        if ui.button("Open read-only").clicked() {
                            self.open_dialog(true);
                        }
                        if ui.button("Open database").clicked() {
                            self.open_dialog(false);
                        }
                        if ui.button("New database").clicked() {
                            self.create_dialog();
                        }
                    });
                });
            });
    }

    fn show_explorer(&mut self, root_ui: &mut egui::Ui) {
        let mut clicked_object: Option<DbObject> = None;
        egui::Panel::left("explorer")
            .default_size(258.0)
            .min_size(210.0)
            .resizable(true)
            .frame(
                egui::Frame::new()
                    .fill(Color32::from_rgb(17, 24, 34))
                    .inner_margin(egui::Margin::same(14)),
            )
            .show(root_ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("EXPLORER")
                            .strong()
                            .size(11.0)
                            .color(Color32::from_rgb(135, 156, 177)),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.small_button("↻").on_hover_text("Refresh schema").clicked() {
                            self.refresh_database();
                        }
                    });
                });
                ui.add_space(8.0);
                ui.add(
                    TextEdit::singleline(&mut self.explorer_filter)
                        .hint_text("Filter objects…")
                        .desired_width(f32::INFINITY),
                );
                ui.add_space(12.0);

                if !self.database.is_open() {
                    ui.label(RichText::new("Your workspace is empty").strong());
                    ui.label(
                        RichText::new("Open a SQLite file or load the demo database.")
                            .small()
                            .color(Color32::from_rgb(140, 157, 177)),
                    );
                    ui.add_space(8.0);
                    if ui.button("Open database").clicked() {
                        self.open_dialog(false);
                    }
                    if ui.button("Load demo workspace").clicked() {
                        self.start_demo();
                    }
                } else {
                    let filter = self.explorer_filter.to_lowercase();
                    for (heading, kind, glyph) in [
                        ("TABLES", "table", "▦"),
                        ("VIEWS", "view", "◫"),
                        ("INDEXES", "index", "⌁"),
                        ("TRIGGERS", "trigger", "ϟ"),
                    ] {
                        let names: Vec<DbObject> = self
                            .objects
                            .iter()
                            .filter(|object| {
                                object.kind == kind && object.name.to_lowercase().contains(&filter)
                            })
                            .cloned()
                            .collect();
                        ui.collapsing(
                            RichText::new(format!("{heading}  {}", names.len()))
                                .size(11.0)
                                .strong()
                                .color(Color32::from_rgb(137, 158, 180)),
                            |ui| {
                                if names.is_empty() {
                                    ui.label(RichText::new("No objects").small().weak());
                                }
                                for object in &names {
                                    let selected = self.selected_object.as_ref().is_some_and(
                                        |current| {
                                            current.name == object.name
                                                && current.kind == object.kind
                                        },
                                    );
                                    let label = format!("{glyph}  {}", object.name);
                                    if ui.selectable_label(selected, label).clicked() {
                                        clicked_object = Some(object.clone());
                                    }
                                }
                            },
                        );
                    }
                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new("WORKSPACE")
                            .size(10.0)
                            .strong()
                            .color(Color32::from_rgb(137, 158, 180)),
                    );
                    if ui
                        .selectable_label(
                            self.active_tab == WorkspaceTab::Query,
                            "⌘  SQL Workbench",
                        )
                        .clicked()
                    {
                        self.active_tab = WorkspaceTab::Query;
                    }
                    if ui
                        .selectable_label(
                            self.active_tab == WorkspaceTab::History,
                            "◷  Query history",
                        )
                        .clicked()
                    {
                        self.active_tab = WorkspaceTab::History;
                    }
                    if ui.button("✓  Integrity check").clicked() {
                        self.run_integrity_check();
                    }
                }

                ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
                    ui.separator();
                    ui.label(
                        RichText::new("OXIDE STUDIO  ·  0.1.0")
                            .size(10.0)
                            .color(Color32::from_rgb(106, 125, 147)),
                    );
                });
            });
        if let Some(object) = clicked_object {
            self.select_object(object);
        }
    }

    fn show_footer(&self, root_ui: &mut egui::Ui) {
        egui::Panel::bottom("footer")
            .frame(
                egui::Frame::new()
                    .fill(Color32::from_rgb(11, 17, 25))
                    .inner_margin(egui::Margin::symmetric(16, 7)),
            )
            .show(root_ui, |ui| {
                ui.horizontal(|ui| {
                    let dot = if self.last_error.is_some() {
                        Color32::from_rgb(236, 104, 104)
                    } else {
                        Color32::from_rgb(91, 207, 172)
                    };
                    ui.label(RichText::new("●").color(dot));
                    ui.label(
                        RichText::new(&self.status_message)
                            .small()
                            .color(Color32::from_rgb(160, 176, 195)),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if let Some(version) = &self.sqlite_version {
                            ui.label(
                                RichText::new(format!("SQLite {version}"))
                                    .small()
                                    .color(Color32::from_rgb(119, 139, 160)),
                            );
                        }
                        ui.label(
                            RichText::new("LOCAL SESSION")
                                .size(9.0)
                                .strong()
                                .color(Color32::from_rgb(91, 207, 172)),
                        );
                    });
                });
                if let Some(error) = &self.last_error {
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(error)
                            .small()
                            .color(Color32::from_rgb(255, 137, 137)),
                    );
                }
            });
    }

    fn show_data_view(&mut self, ui: &mut egui::Ui) {
        let Some(table) = self.selected_table.clone() else {
            ui.centered_and_justified(|ui| ui.label("Select a table or view from the explorer."));
            return;
        };
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.heading(RichText::new(&table).size(22.0).strong());
                ui.label(
                    RichText::new("DATA BROWSER")
                        .size(10.0)
                        .strong()
                        .color(Color32::from_rgb(126, 149, 170)),
                );
            });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.button("Structure").clicked() {
                    self.active_tab = WorkspaceTab::Structure;
                }
                if ui.button("↻ Refresh").clicked() {
                    self.load_table(&table);
                }
                if ui
                    .add_enabled(
                        self.table_offset >= TABLE_PAGE_SIZE,
                        egui::Button::new("← Previous"),
                    )
                    .clicked()
                {
                    self.table_offset = self.table_offset.saturating_sub(TABLE_PAGE_SIZE);
                    self.load_table(&table);
                }
                if ui
                    .add_enabled(
                        self.table_has_next,
                        egui::Button::new("Next →"),
                    )
                    .clicked()
                {
                    self.table_offset += TABLE_PAGE_SIZE;
                    self.load_table(&table);
                }
            });
        });
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("{} columns", self.table_data.columns.len()))
                    .small()
                    .color(Color32::from_rgb(139, 159, 179)),
            );
            ui.separator();
            ui.label(
                RichText::new(format!("{} rows on this page", self.table_data.rows.len()))
                    .small()
                    .color(Color32::from_rgb(139, 159, 179)),
            );
            ui.separator();
            ui.label(
                RichText::new(format!("Offset {}", self.table_offset))
                    .small()
                    .color(Color32::from_rgb(139, 159, 179)),
            );
        });
        ui.add_space(6.0);
        if self.table_data.columns.is_empty() {
            ui.label("This object has no visible columns, or it could not be read as a table.");
            return;
        }
        let columns = self.table_data.columns.clone();
        let rows = self.table_data.rows.clone();
        egui::Frame::new()
            .fill(Color32::from_rgb(12, 19, 28))
            .corner_radius(egui::CornerRadius::same(8))
            .inner_margin(egui::Margin::same(8))
            .show(ui, |ui| {
                ScrollArea::both()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        egui::Grid::new("table-data-grid")
                            .striped(true)
                            .min_col_width(110.0)
                            .spacing([18.0, 9.0])
                            .show(ui, |ui| {
                                for column in &columns {
                                    ui.label(
                                        RichText::new(column)
                                            .strong()
                                            .color(Color32::from_rgb(127, 224, 207)),
                                    );
                                }
                                ui.end_row();
                                for row in &rows {
                                    for value in row {
                                        let text = if value == "NULL" {
                                            RichText::new(value)
                                                .italics()
                                                .color(Color32::from_rgb(128, 145, 164))
                                        } else {
                                            RichText::new(value)
                                                .color(Color32::from_rgb(215, 225, 237))
                                        };
                                        ui.label(text);
                                    }
                                    ui.end_row();
                                }
                            });
                    });
            });
        if rows.is_empty() {
            ui.add_space(14.0);
            ui.label(
                RichText::new("No rows found on this page.")
                    .color(Color32::from_rgb(140, 157, 177)),
            );
        }
    }

    fn show_query_view(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.heading(RichText::new("SQL Workbench").size(22.0));
                ui.label(
                    RichText::new("WRITE · EXECUTE · INSPECT")
                        .size(10.0)
                        .strong()
                        .color(Color32::from_rgb(126, 149, 170)),
                );
            });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.button("Clear").clicked() {
                    self.sql_editor.clear();
                }
                if ui.button("Export CSV").clicked() {
                    self.export_result();
                }
                if ui
                    .add_enabled(self.database.is_open(), egui::Button::new("▶  Run query"))
                    .clicked()
                {
                    self.run_query();
                }
            });
        });
        ui.add_space(10.0);
        ui.label(
            RichText::new("SQL EDITOR")
                .size(10.0)
                .strong()
                .color(Color32::from_rgb(135, 157, 180)),
        );
        egui::Frame::new()
            .fill(Color32::from_rgb(10, 16, 24))
            .corner_radius(egui::CornerRadius::same(8))
            .inner_margin(egui::Margin::same(10))
            .show(ui, |ui| {
                ui.add(
                    TextEdit::multiline(&mut self.sql_editor)
                        .code_editor()
                        .desired_rows(12)
                        .desired_width(f32::INFINITY)
                        .hint_text("-- Write a SQL statement here…"),
                );
            });
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("RESULTS")
                    .size(10.0)
                    .strong()
                    .color(Color32::from_rgb(135, 157, 180)),
            );
            ui.separator();
            if let Some(result) = &self.query_result {
                if let Some(affected) = result.rows_affected {
                    ui.label(RichText::new(format!("{affected} rows affected")).small());
                } else {
                    ui.label(RichText::new(format!("{} rows", result.rows.len())).small());
                }
                ui.label(
                    RichText::new(format!("{} ms", result.elapsed_ms))
                        .small()
                        .color(Color32::from_rgb(127, 224, 207)),
                );
                if result.truncated {
                    ui.label(
                        RichText::new(format!("Limited to {} rows", result.rows.len()))
                            .small()
                            .color(Color32::from_rgb(239, 191, 103)),
                    );
                }
            }
        });
        ui.add_space(6.0);
        if let Some(result) = &self.query_result {
            if !result.columns.is_empty() {
                let columns = result.columns.clone();
                let rows = result.rows.clone();
                egui::Frame::new()
                    .fill(Color32::from_rgb(12, 19, 28))
                    .corner_radius(egui::CornerRadius::same(8))
                    .inner_margin(egui::Margin::same(8))
                    .show(ui, |ui| {
                        ScrollArea::both()
                            .max_height(310.0)
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                egui::Grid::new("query-result-grid")
                                    .striped(true)
                                    .min_col_width(105.0)
                                    .spacing([16.0, 8.0])
                                    .show(ui, |ui| {
                                        for column in &columns {
                                            ui.label(
                                                RichText::new(column)
                                                    .strong()
                                                    .color(Color32::from_rgb(127, 224, 207)),
                                            );
                                        }
                                        ui.end_row();
                                        for row in &rows {
                                            for value in row {
                                                let color = if value == "NULL" {
                                                    Color32::from_rgb(128, 145, 164)
                                                } else {
                                                    Color32::from_rgb(215, 225, 237)
                                                };
                                                ui.label(RichText::new(value).color(color));
                                            }
                                            ui.end_row();
                                        }
                                    });
                            });
                    });
            } else if let Some(affected) = result.rows_affected {
                ui.label(
                    RichText::new(format!(
                        "Statement completed successfully. {affected} row(s) affected."
                    ))
                    .color(Color32::from_rgb(125, 220, 183)),
                );
            }
        } else {
            ui.label(
                RichText::new("Run a statement to see its result here.")
                    .color(Color32::from_rgb(137, 157, 177)),
            );
        }
        ui.add_space(4.0);
        ui.label(
            RichText::new("Prototype note: one SQL statement per run; bind parameters are not exposed in the UI yet. Results are capped at 1,000 rows.")
                .small()
                .color(Color32::from_rgb(112, 131, 152)),
        );
    }

    fn show_structure_view(&mut self, ui: &mut egui::Ui) {
        let Some(object) = self.selected_object.clone() else {
            ui.centered_and_justified(|ui| ui.label("Select a schema object from the explorer."));
            return;
        };
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.heading(RichText::new(&object.name).size(22.0));
                ui.label(
                    RichText::new(format!("{} · SCHEMA INSPECTOR", object.kind.to_uppercase()))
                        .size(10.0)
                        .strong()
                        .color(Color32::from_rgb(126, 149, 170)),
                );
            });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if (object.kind == "table" || object.kind == "view")
                    && ui.button("Open data").clicked()
                {
                    self.active_tab = WorkspaceTab::Data;
                }
            });
        });
        ui.add_space(12.0);
        if object.kind == "table" || object.kind == "view" {
            ui.label(
                RichText::new("COLUMNS")
                    .size(10.0)
                    .strong()
                    .color(Color32::from_rgb(135, 157, 180)),
            );
            let columns = self.table_columns.clone();
            egui::Frame::new()
                .fill(Color32::from_rgb(12, 19, 28))
                .corner_radius(egui::CornerRadius::same(8))
                .inner_margin(egui::Margin::same(8))
                .show(ui, |ui| {
                    egui::Grid::new("schema-columns-grid")
                        .striped(true)
                        .spacing([18.0, 10.0])
                        .show(ui, |ui| {
                            for heading in ["#", "NAME", "TYPE", "NOT NULL", "DEFAULT", "PK"] {
                                ui.label(
                                    RichText::new(heading)
                                        .strong()
                                        .size(10.0)
                                        .color(Color32::from_rgb(127, 224, 207)),
                                );
                            }
                            ui.end_row();
                            for column in &columns {
                                ui.label(column.ordinal.to_string());
                                ui.label(RichText::new(&column.name).strong());
                                ui.label(if column.data_type.is_empty() {
                                    "—"
                                } else {
                                    &column.data_type
                                });
                                ui.label(if column.not_null { "Yes" } else { "No" });
                                ui.label(column.default_value.as_deref().unwrap_or("—"));
                                ui.label(if column.primary_key_order > 0 {
                                    column.primary_key_order.to_string()
                                } else {
                                    "—".to_owned()
                                });
                                ui.end_row();
                            }
                        });
                });
        }
        ui.add_space(14.0);
        ui.label(
            RichText::new("SQL DEFINITION")
                .size(10.0)
                .strong()
                .color(Color32::from_rgb(135, 157, 180)),
        );
        let mut ddl = object
            .sql
            .unwrap_or_else(|| "-- No SQL definition is stored for this object.".to_owned());
        egui::Frame::new()
            .fill(Color32::from_rgb(10, 16, 24))
            .corner_radius(egui::CornerRadius::same(8))
            .inner_margin(egui::Margin::same(10))
            .show(ui, |ui| {
                ui.add(
                    TextEdit::multiline(&mut ddl)
                        .interactive(false)
                        .font(egui::TextStyle::Monospace)
                        .desired_rows(8)
                        .desired_width(f32::INFINITY),
                );
            });
    }

    fn show_history_view(&mut self, ui: &mut egui::Ui) {
        ui.heading(RichText::new("Query history").size(22.0));
        ui.label(
            RichText::new("Recent statements from this session")
                .small()
                .color(Color32::from_rgb(126, 149, 170)),
        );
        ui.add_space(12.0);
        if self.query_history.is_empty() {
            ui.label("No statements have been executed in this session yet.");
            return;
        }
        let mut selected_sql = None;
        ScrollArea::vertical().show(ui, |ui| {
            for (index, sql) in self.query_history.iter().enumerate().rev() {
                egui::Frame::new()
                    .fill(Color32::from_rgb(15, 23, 33))
                    .corner_radius(egui::CornerRadius::same(7))
                    .inner_margin(egui::Margin::same(10))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!("#{:03}", index + 1))
                                    .small()
                                    .color(Color32::from_rgb(111, 208, 193)),
                            );
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if ui.small_button("Load into editor").clicked() {
                                    selected_sql = Some(sql.clone());
                                }
                            });
                        });
                        ui.label(
                            RichText::new(sql)
                                .monospace()
                                .color(Color32::from_rgb(194, 208, 224)),
                        );
                    });
                ui.add_space(7.0);
            }
        });
        if let Some(sql) = selected_sql {
            self.sql_editor = sql;
            self.active_tab = WorkspaceTab::Query;
        }
    }

    fn show_welcome(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(56.0);
            ui.label(RichText::new("◈").size(52.0).color(Color32::from_rgb(113, 224, 204)));
            ui.add_space(10.0);
            ui.heading(RichText::new("A clearer way to work with SQLite").size(28.0).strong());
            ui.label(
                RichText::new("A fast, local-first database workspace built in Rust.")
                    .size(15.0)
                    .color(Color32::from_rgb(147, 166, 187)),
            );
            ui.add_space(30.0);
            egui::Frame::new()
                .fill(Color32::from_rgb(20, 30, 42))
                .corner_radius(egui::CornerRadius::same(12))
                .inner_margin(egui::Margin::same(24))
                .show(ui, |ui| {
                    ui.set_max_width(520.0);
                    ui.heading("Start with a database");
                    ui.label(
                        RichText::new("Open an existing SQLite file, create a new one, or explore the built-in sample workspace.")
                            .color(Color32::from_rgb(151, 170, 190)),
                    );
                    ui.add_space(14.0);
                    ui.horizontal(|ui| {
                        if ui.button("Open database").clicked() {
                            self.open_dialog(false);
                        }
                        if ui.button("Create database").clicked() {
                            self.create_dialog();
                        }
                        if ui.button("Load demo").clicked() {
                            self.start_demo();
                        }
                    });
                });
            ui.add_space(28.0);
            ui.horizontal(|ui| {
                for (title, description) in [
                    ("Explore", "Tables, views, indexes and triggers"),
                    ("Query", "Execute SQL and inspect results"),
                    ("Export", "Save query results as CSV"),
                ] {
                    egui::Frame::new()
                        .fill(Color32::from_rgb(17, 25, 36))
                        .corner_radius(egui::CornerRadius::same(8))
                        .inner_margin(egui::Margin::same(14))
                        .show(ui, |ui| {
                            ui.set_min_width(150.0);
                            ui.label(
                                RichText::new(title)
                                    .strong()
                                    .color(Color32::from_rgb(127, 224, 207)),
                            );
                            ui.label(
                                RichText::new(description)
                                    .small()
                                    .color(Color32::from_rgb(143, 162, 182)),
                            );
                        });
                }
            });
        });
    }

    fn show_about(&mut self, ctx: &egui::Context) {
        if self.show_about {
            egui::Window::new("About Oxide Studio")
                .collapsible(false)
                .resizable(false)
                .open(&mut self.show_about)
                .show(ctx, |ui| {
                    ui.heading(
                        RichText::new("Oxide Studio")
                            .color(Color32::from_rgb(113, 224, 204)),
                    );
                    ui.label("Version 0.1.0 · Prototype");
                    ui.label("A native Rust workspace for SQLite database exploration.");
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new("Local-first · Bundled SQLite · MIT License")
                            .small()
                            .weak(),
                    );
                });
        }
    }
}

impl eframe::App for OxideApp {
    fn ui(&mut self, root_ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = root_ui.ctx().clone();
        if ctx.input(|input| input.modifiers.ctrl && input.key_pressed(egui::Key::Enter)) {
            self.run_query();
            self.active_tab = WorkspaceTab::Query;
        }

        self.show_header(root_ui);
        self.show_footer(root_ui);
        self.show_explorer(root_ui);

        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(Color32::from_rgb(15, 21, 30))
                    .inner_margin(egui::Margin::same(20)),
            )
            .show(root_ui, |ui| match self.active_tab {
                WorkspaceTab::Welcome => self.show_welcome(ui),
                WorkspaceTab::Data => self.show_data_view(ui),
                WorkspaceTab::Query => self.show_query_view(ui),
                WorkspaceTab::Structure => self.show_structure_view(ui),
                WorkspaceTab::History => self.show_history_view(ui),
            });

        self.show_about(&ctx);
    }
}
