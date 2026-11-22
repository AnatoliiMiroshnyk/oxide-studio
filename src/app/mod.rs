use std::path::PathBuf;

use anyhow::{anyhow, Result};
use rusqlite::types::Value;
use eframe::egui::{self, Align, Color32, Layout, RichText, ScrollArea, TextEdit};
use rfd::FileDialog;

use crate::{
    db::{ColumnInfo, ColumnSpec, DatabaseManager, DbObject, QueryResult, RowIdentity, SqlValue, TableData, TransactionMode},
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RowEditorMode { Insert, Update }

#[derive(Debug, Clone)]
struct RowField {
    name: String,
    data_type: String,
    text: String,
    initial_text: String,
    is_null: bool,
    initial_is_null: bool,
    include: bool,
    editable: bool,
}

#[derive(Debug, Clone)]
struct RowEditorState {
    mode: RowEditorMode,
    table: String,
    fields: Vec<RowField>,
    identity: Option<RowIdentity>,
}

#[derive(Debug, Clone)]
struct ColumnDraft {
    name: String,
    data_type: String,
    not_null: bool,
    primary_key: bool,
}

#[derive(Debug, Clone)]
enum DestructiveAction {
    DeleteRow { table: String, identity: RowIdentity },
    DropTable { table: String },
    DropColumn { table: String, column: String },
    RollbackTransaction,
}

#[derive(Debug, Clone)]
struct PendingDestructive {
    action: DestructiveAction,
    title: String,
    warning: String,
    confirm_token: String,
    typed_confirmation: String,
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
    selected_row_index: Option<usize>,
    row_editor: Option<RowEditorState>,
    pending_destructive: Option<PendingDestructive>,
    show_create_table: bool,
    create_table_name: String,
    create_table_columns: Vec<ColumnDraft>,
    schema_new_column_name: String,
    schema_new_column_type: String,
    schema_new_column_not_null: bool,
    schema_default_value: String,
    schema_default_enabled: bool,
    schema_selected_column: String,
    schema_rename_table_value: String,
    schema_rename_column_value: String,
    transaction_mode: TransactionMode,
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
            selected_row_index: None,
            row_editor: None,
            pending_destructive: None,
            show_create_table: false,
            create_table_name: String::new(),
            create_table_columns: vec![
                ColumnDraft { name: "id".into(), data_type: "INTEGER".into(), not_null: true, primary_key: true },
                ColumnDraft { name: "name".into(), data_type: "TEXT".into(), not_null: false, primary_key: false },
            ],
            schema_new_column_name: String::new(),
            schema_new_column_type: "TEXT".into(),
            schema_new_column_not_null: false,
            schema_default_value: String::new(),
            schema_default_enabled: false,
            schema_selected_column: String::new(),
            schema_rename_table_value: String::new(),
            schema_rename_column_value: String::new(),
            transaction_mode: TransactionMode::Immediate,
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
        if self.database.is_in_transaction() {
            self.set_error("Commit or roll back the active transaction before opening another database");
            return;
        }
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
                self.selected_row_index = None;
                self.row_editor = None;
                self.pending_destructive = None;
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
        if self.database.is_in_transaction() {
            self.set_error("Commit or roll back the active transaction before switching databases");
            return;
        }
        match self.database.open_demo() {
            Ok(()) => {
                self.status_message = "Demo workspace connected · in-memory database".to_owned();
                self.last_error = None;
                self.selected_table = None;
                self.selected_object = None;
                self.selected_row_index = None;
                self.row_editor = None;
                self.pending_destructive = None;
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
        if self.selected_table.is_none() {
            if let Some(object) = self.objects.iter().find(|object| object.kind == "table") {
                self.selected_table = Some(object.name.clone());
                self.selected_object = Some(object.clone());
            }
        } else if let Some(name) = self.selected_table.as_ref() {
            self.selected_object = self.objects.iter()
                .find(|object| object.name == *name && (object.kind == "table" || object.kind == "view"))
                .cloned();
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
            self.selected_row_index = None;
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
        // A refresh invalidates the row-index selection; never carry an index onto a new snapshot.
        self.selected_row_index = None;
        match self.database.table_data(table, TABLE_PAGE_SIZE + 1, self.table_offset) {
            Ok(mut data) => {
                self.table_has_next = data.rows.len() > TABLE_PAGE_SIZE;
                data.rows.truncate(TABLE_PAGE_SIZE);
                data.raw_rows.truncate(TABLE_PAGE_SIZE);
                data.row_identities.truncate(TABLE_PAGE_SIZE);
                self.table_data = data;
                self.table_columns = self.database.table_columns(table).unwrap_or_default();
                if !self.table_columns.iter().any(|column| column.name == self.schema_selected_column) {
                    self.schema_selected_column = self.table_columns.first().map(|column| column.name.clone()).unwrap_or_default();
                }
                if self.selected_row_index.is_some_and(|index| index >= self.table_data.rows.len()) {
                    self.selected_row_index = None;
                }
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

    fn open_insert_dialog(&mut self, table: &str) {
        if self.database.is_read_only() { return; }
        match self.database.table_columns(table) {
            Ok(columns) => {
                let fields = columns.into_iter().map(|column| RowField {
                    name: column.name,
                    data_type: column.data_type,
                    text: String::new(),
                    initial_text: String::new(),
                    is_null: true,
                    initial_is_null: true,
                    include: column.hidden == 0 && column.default_value.is_none() && column.primary_key_order == 0,
                    editable: column.hidden == 0,
                }).collect();
                self.row_editor = Some(RowEditorState { mode: RowEditorMode::Insert, table: table.to_owned(), fields, identity: None });
            }
            Err(error) => self.set_error(error),
        }
    }

    fn open_edit_dialog(&mut self, table: &str, index: usize) {
        if self.database.is_read_only() { return; }
        let Some(values) = self.table_data.raw_rows.get(index) else { return; };
        let Some(Some(identity)) = self.table_data.row_identities.get(index) else {
            self.status_message = "This row has no safe identity; editing is disabled for this table".to_owned();
            return;
        };
        let columns = self.table_columns.clone();
        let fields = columns.into_iter().zip(values.iter()).map(|(column, value)| {
            let text = sql_value_to_input(value);
            RowField {
                name: column.name,
                data_type: column.data_type,
                initial_text: text.clone(),
                text,
                is_null: matches!(value, Value::Null),
                initial_is_null: matches!(value, Value::Null),
                include: true,
                editable: column.hidden == 0,
            }
        }).collect();
        self.row_editor = Some(RowEditorState {
            mode: RowEditorMode::Update,
            table: table.to_owned(),
            fields,
            identity: Some(identity.clone()),
        });
    }

    fn save_row_editor(&mut self) {
        let Some(editor) = self.row_editor.as_ref().cloned() else { return; };
        let mut values: Vec<(String, SqlValue)> = Vec::new();
        for field in &editor.fields {
            if !field.editable { continue; }
            if editor.mode == RowEditorMode::Insert && !field.include { continue; }
            if editor.mode == RowEditorMode::Update
                && field.text == field.initial_text
                && field.is_null == field.initial_is_null
            {
                continue;
            }
            match field_to_sql_value(field) {
                Ok(value) => values.push((field.name.clone(), value)),
                Err(error) => {
                    self.set_error(format!("{}: {error}", field.name));
                    return;
                }
            }
        }
        let result = match editor.mode {
            RowEditorMode::Insert => self.database.insert_row(&editor.table, &values).map(|count| ("Inserted", count)),
            RowEditorMode::Update => {
                if values.is_empty() {
                    self.status_message = "No values were changed".to_owned();
                    self.row_editor = None;
                    return;
                }
                let Some(identity) = editor.identity.as_ref() else {
                    self.set_error("The selected row has no stable identity");
                    return;
                };
                self.database.update_row(&editor.table, identity, &values).map(|count| ("Updated", count))
            }
        };
        match result {
            Ok((verb, count)) => {
                self.row_editor = None;
                self.selected_row_index = None;
                self.status_message = format!("{verb} {count} row(s) in {}", editor.table);
                self.refresh_database();
            }
            Err(error) => self.set_error(error),
        }
    }

    fn request_delete_selected(&mut self) {
        let Some(table) = self.selected_table.clone() else { return; };
        let Some(index) = self.selected_row_index else { return; };
        let Some(Some(identity)) = self.table_data.row_identities.get(index) else {
            self.status_message = "Delete is unavailable because this row has no safe identity".to_owned();
            return;
        };
        let summary = self.table_data.rows.get(index).map(|row| row.iter().take(3).cloned().collect::<Vec<_>>().join(" · ")).unwrap_or_default();
        self.pending_destructive = Some(PendingDestructive {
            action: DestructiveAction::DeleteRow { table: table.clone(), identity: identity.clone() },
            title: "Delete selected row?".to_owned(),
            warning: format!("This permanently deletes one row from {table}. Preview: {summary}. The operation is not recoverable after commit."),
            confirm_token: "DELETE".to_owned(),
            typed_confirmation: String::new(),
        });
    }

    fn request_drop_table(&mut self, table: &str) {
        self.pending_destructive = Some(PendingDestructive {
            action: DestructiveAction::DropTable { table: table.to_owned() },
            title: format!("Drop table {table}?"),
            warning: "This removes the table and all of its rows. Dependent views, triggers or foreign-key relationships may prevent the operation or be affected by SQLite's schema rules. Create a backup before proceeding.".to_owned(),
            confirm_token: table.to_owned(),
            typed_confirmation: String::new(),
        });
    }

    fn request_drop_column(&mut self, table: &str, column: &str) {
        self.pending_destructive = Some(PendingDestructive {
            action: DestructiveAction::DropColumn { table: table.to_owned(), column: column.to_owned() },
            title: format!("Drop column {column}?"),
            warning: format!("This permanently removes column {column} and every value stored in it. Indexes, constraints, generated expressions or dependent objects may cause SQLite to reject the operation. Create a backup first."),
            confirm_token: column.to_owned(),
            typed_confirmation: String::new(),
        });
    }

    fn perform_destructive_action(&mut self, action: DestructiveAction) {
        let result = match &action {
            DestructiveAction::DeleteRow { table, identity } => self.database.delete_row(table, identity),
            DestructiveAction::DropTable { table } => self.database.drop_table(table).map(|_| 1),
            DestructiveAction::DropColumn { table, column } => self.database.drop_column(table, column).map(|_| 1),
            DestructiveAction::RollbackTransaction => self.database.rollback_transaction().map(|_| 1),
        };
        match result {
            Ok(affected) => {
                match action {
                    DestructiveAction::DeleteRow { table, .. } => self.status_message = format!("Deleted {affected} row(s) from {table}"),
                    DestructiveAction::DropTable { table } => {
                        self.selected_table = None;
                        self.selected_object = None;
                        self.selected_row_index = None;
                        self.status_message = format!("Dropped table {table}");
                    }
                    DestructiveAction::DropColumn { table, column } => self.status_message = format!("Dropped column {column} from {table}"),
                    DestructiveAction::RollbackTransaction => self.status_message = "Transaction rolled back; uncommitted changes were discarded".to_owned(),
                }
                self.selected_row_index = None;
                self.refresh_database();
            }
            Err(error) => self.set_error(error),
        }
    }

    fn create_table_from_dialog(&mut self) {
        let columns = self.create_table_columns.iter().map(|column| ColumnSpec {
            name: column.name.trim().to_owned(),
            data_type: column.data_type.trim().to_owned(),
            not_null: column.not_null,
            primary_key: column.primary_key,
        }).collect::<Vec<_>>();
        match self.database.create_table(self.create_table_name.trim(), &columns) {
            Ok(()) => {
                let name = self.create_table_name.trim().to_owned();
                self.show_create_table = false;
                self.create_table_name.clear();
                self.status_message = format!("Created table {name}");
                self.refresh_database();
                self.select_named_object(&name);
            }
            Err(error) => self.set_error(error),
        }
    }

    fn show_row_editor(&mut self, ctx: &egui::Context) {
        let mut open = true;
        let mut save = false;
        let mut cancel = false;
        if let Some(editor) = self.row_editor.as_mut() {
            let title = if editor.mode == RowEditorMode::Insert { format!("Insert row · {}", editor.table) } else { format!("Edit row · {}", editor.table) };
            egui::Window::new(title).collapsible(false).resizable(true).default_width(560.0).open(&mut open).show(ctx, |ui| {
                ui.label("Values are bound as SQLite parameters. Use NULL explicitly; an empty text field is an empty string.");
                ui.add_space(8.0);
                ScrollArea::vertical().max_height(470.0).show(ui, |ui| {
                    for field in &mut editor.fields {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(&field.name).strong());
                                ui.label(RichText::new(if field.data_type.is_empty() { "ANY" } else { &field.data_type }).small().weak());
                                if !field.editable { ui.label(RichText::new("GENERATED").small().color(Color32::from_rgb(239, 191, 103))); }
                            });
                            ui.horizontal(|ui| {
                                if editor.mode == RowEditorMode::Insert {
                                    ui.add_enabled(field.editable, egui::Checkbox::new(&mut field.include, "Include"));
                                }
                                let value_enabled = field.editable && (editor.mode != RowEditorMode::Insert || field.include);
                                ui.add_enabled(value_enabled, egui::Checkbox::new(&mut field.is_null, "NULL"));
                                ui.add_enabled(value_enabled && !field.is_null, TextEdit::singleline(&mut field.text).desired_width(f32::INFINITY).hint_text("Value (BLOB: hex:0011…)"));
                            });
                        });
                    }
                });
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button(if editor.mode == RowEditorMode::Insert { "Insert row" } else { "Save changes" }).clicked() { save = true; }
                    if ui.button("Cancel").clicked() { cancel = true; }
                });
            });
        }
        if cancel || !open { self.row_editor = None; }
        if save { self.save_row_editor(); }
    }

    fn show_destructive_confirmation(&mut self, ctx: &egui::Context) {
        let mut open = true;
        let mut confirm = false;
        let mut cancel = false;
        if let Some(pending) = self.pending_destructive.as_mut() {
            egui::Window::new(&pending.title).collapsible(false).resizable(false).open(&mut open).show(ctx, |ui| {
                ui.label(RichText::new(&pending.warning).color(Color32::from_rgb(245, 178, 112)));
                ui.add_space(8.0);
                ui.label(format!("Type {} exactly to confirm:", pending.confirm_token));
                ui.add(TextEdit::singleline(&mut pending.typed_confirmation).desired_width(320.0));
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.add_enabled(pending.typed_confirmation == pending.confirm_token, egui::Button::new("Confirm destructive action")).clicked() { confirm = true; }
                    if ui.button("Cancel").clicked() { cancel = true; }
                });
            });
        }
        if cancel || !open { self.pending_destructive = None; }
        if confirm && let Some(pending) = self.pending_destructive.take() {
            self.perform_destructive_action(pending.action);
        }
    }

    fn show_create_table_dialog(&mut self, ctx: &egui::Context) {
        let mut open = true;
        let mut create = false;
        let mut cancel = false;
        if self.show_create_table {
            egui::Window::new("Create table").collapsible(false).resizable(true).default_width(620.0).open(&mut open).show(ctx, |ui| {
                ui.label("Define columns and an optional single or composite primary key.");
                ui.horizontal(|ui| {
                    ui.label("Table name");
                    ui.add(TextEdit::singleline(&mut self.create_table_name).desired_width(260.0));
                });
                ui.separator();
                let mut remove_at = None;
                ScrollArea::vertical().max_height(360.0).show(ui, |ui| {
                    for (index, column) in self.create_table_columns.iter_mut().enumerate() {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(format!("Column {}", index + 1));
                                ui.add(TextEdit::singleline(&mut column.name).desired_width(150.0).hint_text("name"));
                                ui.add(TextEdit::singleline(&mut column.data_type).desired_width(120.0).hint_text("TEXT"));
                                ui.checkbox(&mut column.not_null, "NOT NULL");
                                ui.checkbox(&mut column.primary_key, "PK");
                                if ui.small_button("Remove").clicked() { remove_at = Some(index); }
                            });
                        });
                    }
                });
                if let Some(index) = remove_at && self.create_table_columns.len() > 1 {
                    self.create_table_columns.remove(index);
                }
                if ui.button("+ Add column").clicked() {
                    self.create_table_columns.push(ColumnDraft { name: String::new(), data_type: "TEXT".into(), not_null: false, primary_key: false });
                }
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("Create table").clicked() { create = true; }
                    if ui.button("Cancel").clicked() { cancel = true; }
                });
            });
        }
        if cancel || !open { self.show_create_table = false; }
        if create { self.create_table_from_dialog(); }
    }

    fn show_transaction_controls(&mut self, ui: &mut egui::Ui) {
        let active = self.database.is_in_transaction();
        ui.separator();
        if active {
            ui.label(RichText::new("TRANSACTION ACTIVE").small().strong().color(Color32::from_rgb(239, 191, 103)));
            if ui.button("Commit").clicked() {
                match self.database.commit_transaction() {
                    Ok(()) => self.status_message = "Transaction committed".to_owned(),
                    Err(error) => self.set_error(error),
                }
                self.refresh_database();
            }
            if ui.button("Rollback…").clicked() {
                self.pending_destructive = Some(PendingDestructive {
                    action: DestructiveAction::RollbackTransaction,
                    title: "Roll back the active transaction?".to_owned(),
                    warning: "Every write made since BEGIN will be discarded. This cannot be undone after rollback.".to_owned(),
                    confirm_token: "ROLLBACK".to_owned(),
                    typed_confirmation: String::new(),
                });
            }
        } else {
            ui.add_enabled_ui(self.database.is_open() && !self.database.is_read_only(), |ui| {
                egui::ComboBox::from_id_salt("transaction-mode")
                    .selected_text(match self.transaction_mode { TransactionMode::Deferred => "Deferred", TransactionMode::Immediate => "Immediate", TransactionMode::Exclusive => "Exclusive" })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.transaction_mode, TransactionMode::Deferred, "Deferred");
                        ui.selectable_value(&mut self.transaction_mode, TransactionMode::Immediate, "Immediate");
                        ui.selectable_value(&mut self.transaction_mode, TransactionMode::Exclusive, "Exclusive");
                    });
            });
            if ui.add_enabled(self.database.is_open() && !self.database.is_read_only(), egui::Button::new("Begin transaction")).clicked() {
                match self.database.begin_transaction(self.transaction_mode) {
                    Ok(()) => self.status_message = format!("{} transaction started", match self.transaction_mode { TransactionMode::Deferred => "Deferred", TransactionMode::Immediate => "Immediate", TransactionMode::Exclusive => "Exclusive" }),
                    Err(error) => self.set_error(error),
                }
            }
        }
    }

    fn add_selected_column(&mut self, table: &str) {
        let default = if self.schema_default_enabled { Some(Value::Text(self.schema_default_value.clone())) } else { None };
        match self.database.add_column(table, &self.schema_new_column_name, &self.schema_new_column_type, self.schema_new_column_not_null, default.as_ref()) {
            Ok(()) => {
                self.status_message = format!("Added column {}", self.schema_new_column_name);
                self.schema_new_column_name.clear();
                self.schema_default_value.clear();
                self.schema_default_enabled = false;
                self.refresh_database();
            }
            Err(error) => self.set_error(error),
        }
    }

    fn rename_selected_column(&mut self, table: &str) {
        if self.schema_selected_column.is_empty() { return; }
        match self.database.rename_column(table, &self.schema_selected_column, &self.schema_rename_column_value) {
            Ok(()) => {
                self.status_message = format!("Renamed column {}", self.schema_selected_column);
                self.schema_selected_column = self.schema_rename_column_value.clone();
                self.schema_rename_column_value.clear();
                self.refresh_database();
            }
            Err(error) => self.set_error(error),
        }
    }

    fn rename_selected_table(&mut self, table: &str, new_name: &str) {
        match self.database.rename_table(table, new_name) {
            Ok(()) => {
                let name = new_name.to_owned();
                self.selected_table = Some(name.clone());
                self.status_message = format!("Renamed table to {name}");
                self.schema_rename_table_value.clear();
                self.refresh_database();
                self.select_named_object(&name);
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
                        self.show_transaction_controls(ui);
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
                        if ui.add_enabled(self.database.is_open() && !self.database.is_read_only(), egui::Button::new("+"))
                            .on_hover_text("Create table").clicked()
                        {
                            self.show_create_table = true;
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
                        RichText::new("OXIDE STUDIO  ·  0.2.0")
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
                let writable_table = self.selected_object.as_ref().is_some_and(|object| object.kind == "table")
                    && !self.database.is_read_only();
                let selected_identity = self.selected_row_index
                    .and_then(|index| self.table_data.row_identities.get(index))
                    .is_some_and(Option::is_some);
                if ui.button("Structure").clicked() {
                    self.active_tab = WorkspaceTab::Structure;
                }
                if ui.add_enabled(writable_table && selected_identity, egui::Button::new("Delete selected")).clicked() {
                    self.request_delete_selected();
                }
                if ui.add_enabled(writable_table && selected_identity, egui::Button::new("Edit selected")).clicked()
                    && let Some(index) = self.selected_row_index
                {
                    self.open_edit_dialog(&table, index);
                }
                if ui.add_enabled(writable_table, egui::Button::new("+ New row")).clicked() {
                    self.open_insert_dialog(&table);
                }
                if ui.button("↻ Refresh").clicked() {
                    self.load_table(&table);
                }
                if ui.add_enabled(self.table_offset >= TABLE_PAGE_SIZE, egui::Button::new("← Previous")).clicked() {
                    self.table_offset = self.table_offset.saturating_sub(TABLE_PAGE_SIZE);
                    self.selected_row_index = None;
                    self.load_table(&table);
                }
                if ui.add_enabled(self.table_has_next, egui::Button::new("Next →")).clicked() {
                    self.table_offset += TABLE_PAGE_SIZE;
                    self.selected_row_index = None;
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
                                ui.label(RichText::new("ROW").strong().color(Color32::from_rgb(127, 224, 207)));
                                for column in &columns {
                                    ui.label(
                                        RichText::new(column)
                                            .strong()
                                            .color(Color32::from_rgb(127, 224, 207)),
                                    );
                                }
                                ui.end_row();
                                for (row_index, row) in rows.iter().enumerate() {
                                    let selected = self.selected_row_index == Some(row_index);
                                    if ui.selectable_label(selected, format!("{}", self.table_offset + row_index + 1)).clicked() {
                                        self.selected_row_index = Some(row_index);
                                    }
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
        } else if let Some(index) = self.selected_row_index {
            if let Some(Some(identity)) = self.table_data.row_identities.get(index) {
                let description = match identity {
                    RowIdentity::PrimaryKey(_) => "Selected row is identified by its declared primary key.",
                    RowIdentity::RowId { .. } => "Selected row is identified by an internal rowid fallback.",
                };
                ui.label(RichText::new(description).small().weak());
            } else {
                ui.label(RichText::new("Selected row has no safe edit identity; use read-only browsing or add a declared primary key.").small().color(Color32::from_rgb(239, 191, 103)));
            }
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
            RichText::new("One statement per run · results capped at 1,000 rows · use the transaction controls in the top bar to group writes.")
                .small()
                .color(Color32::from_rgb(112, 131, 152)),
        );
        ui.label(
            RichText::new("Advanced SQL runs directly on the active connection. Review destructive DDL/DML carefully; confirmation dialogs apply to the guided editor workflows.")
                .small()
                .color(Color32::from_rgb(239, 191, 103)),
        );
    }

    fn show_structure_view(&mut self, ui: &mut egui::Ui) {
        let Some(object) = self.selected_object.clone() else {
            ui.centered_and_justified(|ui| ui.label("Select a schema object from the explorer."));
            return;
        };
        let mut rename_table_to: Option<String> = None;
        let mut add_column_requested = false;
        let mut rename_column_requested = false;
        let mut drop_table_requested = false;
        let mut drop_column_requested = false;
        let columns_for_tools = self.table_columns.clone();
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
        if object.kind == "table" && !self.database.is_read_only() {
            egui::Frame::new()
                .fill(Color32::from_rgb(17, 27, 38))
                .corner_radius(egui::CornerRadius::same(8))
                .inner_margin(egui::Margin::same(12))
                .show(ui, |ui| {
                    ui.label(RichText::new("SCHEMA EDITOR").strong().size(10.0).color(Color32::from_rgb(127, 224, 207)));
                    ui.horizontal(|ui| {
                        ui.label("Rename table");
                        ui.add(TextEdit::singleline(&mut self.schema_rename_table_value).desired_width(180.0).hint_text("new table name"));
                        if ui.add_enabled(!self.schema_rename_table_value.trim().is_empty(), egui::Button::new("Rename")).clicked() {
                            rename_table_to = Some(self.schema_rename_table_value.trim().to_owned());
                        }
                        if ui.button("Drop table…").clicked() { drop_table_requested = true; }
                    });
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label("Add column");
                        ui.add(TextEdit::singleline(&mut self.schema_new_column_name).desired_width(130.0).hint_text("column name"));
                        ui.add(TextEdit::singleline(&mut self.schema_new_column_type).desired_width(100.0).hint_text("TEXT"));
                        ui.checkbox(&mut self.schema_new_column_not_null, "NOT NULL");
                        ui.checkbox(&mut self.schema_default_enabled, "Default");
                        ui.add_enabled(self.schema_default_enabled, TextEdit::singleline(&mut self.schema_default_value).desired_width(110.0).hint_text("literal value"));
                        if ui.add_enabled(!self.schema_new_column_name.trim().is_empty(), egui::Button::new("Add")).clicked() { add_column_requested = true; }
                    });
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label("Rename / drop column");
                        egui::ComboBox::from_id_salt("schema-column-select")
                            .selected_text(if self.schema_selected_column.is_empty() { "Select column" } else { self.schema_selected_column.as_str() })
                            .show_ui(ui, |ui| {
                                for column in &columns_for_tools {
                                    ui.selectable_value(&mut self.schema_selected_column, column.name.clone(), &column.name);
                                }
                            });
                        ui.add(TextEdit::singleline(&mut self.schema_rename_column_value).desired_width(130.0).hint_text("new column name"));
                        if ui.add_enabled(!self.schema_selected_column.is_empty() && !self.schema_rename_column_value.trim().is_empty(), egui::Button::new("Rename column")).clicked() { rename_column_requested = true; }
                        if ui.add_enabled(!self.schema_selected_column.is_empty(), egui::Button::new("Drop column…")).clicked() { drop_column_requested = true; }
                    });
                    ui.label(RichText::new("Defaults are stored as literal values. SQLite validates constraints and dependencies; unsupported ALTER operations fail without committing a partial change.").small().weak());
                });
        }
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
        if let Some(new_name) = rename_table_to {
            self.rename_selected_table(&object.name, &new_name);
            self.schema_rename_table_value.clear();
        }
        if add_column_requested { self.add_selected_column(&object.name); }
        if rename_column_requested { self.rename_selected_column(&object.name); }
        if drop_table_requested { self.request_drop_table(&object.name); }
        if drop_column_requested {
            let column = self.schema_selected_column.clone();
            self.request_drop_column(&object.name, &column);
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
                    ui.label("Version 0.2.0 · Data editing and transaction management");
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

fn sql_value_to_input(value: &SqlValue) -> String {
    match value {
        Value::Null => String::new(),
        Value::Integer(value) => value.to_string(),
        Value::Real(value) => value.to_string(),
        Value::Text(value) => value.clone(),
        Value::Blob(value) => format!("hex:{}", value.iter().map(|byte| format!("{byte:02X}")).collect::<String>()),
    }
}

fn field_to_sql_value(field: &RowField) -> Result<SqlValue> {
    if field.is_null { return Ok(Value::Null); }
    if let Some(hex) = field.text.strip_prefix("hex:") {
        if hex.len() % 2 != 0 { return Err(anyhow!("BLOB hex input must contain an even number of characters")); }
        let bytes = (0..hex.len()).step_by(2).map(|index| {
            u8::from_str_radix(&hex[index..index + 2], 16)
                .map_err(|_| anyhow!("BLOB value contains a non-hexadecimal character"))
        }).collect::<Result<Vec<_>>>()?;
        return Ok(Value::Blob(bytes));
    }

    let kind = field.data_type.to_ascii_uppercase();
    let value = field.text.clone();
    if kind.contains("INT") {
        return value.trim().parse::<i64>()
            .map(Value::Integer)
            .map_err(|_| anyhow!("expected a valid 64-bit integer"));
    }
    if kind.contains("REAL") || kind.contains("FLOA") || kind.contains("DOUB") {
        let parsed = value.trim().parse::<f64>().map_err(|_| anyhow!("expected a valid number"))?;
        if !parsed.is_finite() { return Err(anyhow!("non-finite numbers are not supported")); }
        return Ok(Value::Real(parsed));
    }
    if kind.contains("NUMERIC") || kind.contains("DECIMAL") || kind.contains("BOOLEAN") || kind.contains("BOOL") {
        if let Ok(integer) = value.trim().parse::<i64>() { return Ok(Value::Integer(integer)); }
        if let Ok(real) = value.trim().parse::<f64>() && real.is_finite() {
            return Ok(Value::Real(real));
        }
    }
    if kind.contains("BLOB") {
        return Err(anyhow!("BLOB values must use the hex: prefix"));
    }
    Ok(Value::Text(value))
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
        self.show_row_editor(&ctx);
        self.show_destructive_confirmation(&ctx);
        self.show_create_table_dialog(&ctx);
    }
}

#[cfg(test)]
mod editing_tests {
    use super::*;

    fn field(data_type: &str, text: &str, is_null: bool) -> RowField {
        RowField {
            name: "value".to_owned(),
            data_type: data_type.to_owned(),
            text: text.to_owned(),
            initial_text: String::new(),
            is_null,
            initial_is_null: true,
            include: true,
            editable: true,
        }
    }

    #[test]
    fn null_and_empty_text_are_distinct() {
        assert_eq!(field_to_sql_value(&field("TEXT", "", true)).unwrap(), Value::Null);
        assert_eq!(field_to_sql_value(&field("TEXT", "", false)).unwrap(), Value::Text(String::new()));
    }

    #[test]
    fn numeric_fields_are_parsed_and_invalid_values_are_rejected() {
        assert_eq!(field_to_sql_value(&field("INTEGER", "42", false)).unwrap(), Value::Integer(42));
        assert_eq!(field_to_sql_value(&field("REAL", "3.5", false)).unwrap(), Value::Real(3.5));
        assert!(field_to_sql_value(&field("INTEGER", "42x", false)).is_err());
        assert!(field_to_sql_value(&field("REAL", "NaN", false)).is_err());
    }

    #[test]
    fn blob_fields_accept_valid_hex_only() {
        assert_eq!(field_to_sql_value(&field("BLOB", "hex:00a1FF", false)).unwrap(), Value::Blob(vec![0, 0xA1, 0xFF]));
        assert!(field_to_sql_value(&field("BLOB", "hex:0", false)).is_err());
        assert!(field_to_sql_value(&field("BLOB", "plain text", false)).is_err());
    }
}
