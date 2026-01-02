use crate::models::{Collection, HttpResponse, Project, Request, Workspace};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    Sidebar,
    RequestEditor,
    ResponseViewer,
}

impl Panel {
    pub fn next(&self) -> Panel {
        match self {
            Panel::Sidebar => Panel::RequestEditor,
            Panel::RequestEditor => Panel::ResponseViewer,
            Panel::ResponseViewer => Panel::Sidebar,
        }
    }

    pub fn prev(&self) -> Panel {
        match self {
            Panel::Sidebar => Panel::ResponseViewer,
            Panel::RequestEditor => Panel::Sidebar,
            Panel::ResponseViewer => Panel::RequestEditor,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Editing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarItem {
    Project(usize),
    Collection(usize, usize),
    Request(usize, usize, usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorTab {
    Url,
    Headers,
    Body,
}

impl EditorTab {
    pub fn next(&self) -> EditorTab {
        match self {
            EditorTab::Url => EditorTab::Headers,
            EditorTab::Headers => EditorTab::Body,
            EditorTab::Body => EditorTab::Url,
        }
    }

    pub fn prev(&self) -> EditorTab {
        match self {
            EditorTab::Url => EditorTab::Body,
            EditorTab::Headers => EditorTab::Url,
            EditorTab::Body => EditorTab::Headers,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorField {
    Method,
    Url,
    HeaderKey,
    HeaderValue,
    Body,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogType {
    NewProject,
    NewCollection,
    NewRequest,
    DeleteConfirm,
    RenameItem,
}

pub struct App {
    pub workspace: Workspace,
    pub active_panel: Panel,
    pub input_mode: InputMode,

    // Sidebar state
    pub sidebar_items: Vec<SidebarItem>,
    pub sidebar_selected: usize,
    pub expanded_projects: Vec<usize>,
    pub expanded_collections: Vec<(usize, usize)>,

    // Request editor state
    pub editor_tab: EditorTab,
    pub editor_field: EditorField,
    pub current_request: Option<Request>,
    pub current_request_path: Option<(usize, usize, usize)>,

    // Input buffers
    pub url_input: String,
    pub body_input: String,
    pub header_key_input: String,
    pub header_value_input: String,
    pub editing_header_index: Option<usize>,
    pub headers_list: Vec<(String, String)>,

    // Response state
    pub response: Option<HttpResponse>,
    pub response_scroll: u16,
    pub is_loading: bool,
    pub error_message: Option<String>,

    // Dialog state
    pub dialog: Option<DialogType>,
    pub dialog_input: String,

    // App state
    pub should_quit: bool,
    pub needs_save: bool,
}

impl App {
    pub fn new(workspace: Workspace) -> Self {
        let mut app = Self {
            workspace,
            active_panel: Panel::Sidebar,
            input_mode: InputMode::Normal,
            sidebar_items: Vec::new(),
            sidebar_selected: 0,
            expanded_projects: Vec::new(),
            expanded_collections: Vec::new(),
            editor_tab: EditorTab::Url,
            editor_field: EditorField::Url,
            current_request: None,
            current_request_path: None,
            url_input: String::new(),
            body_input: String::new(),
            header_key_input: String::new(),
            header_value_input: String::new(),
            editing_header_index: None,
            headers_list: Vec::new(),
            response: None,
            response_scroll: 0,
            is_loading: false,
            error_message: None,
            dialog: None,
            dialog_input: String::new(),
            should_quit: false,
            needs_save: false,
        };
        app.rebuild_sidebar_items();
        app
    }

    pub fn rebuild_sidebar_items(&mut self) {
        self.sidebar_items.clear();

        for (pi, project) in self.workspace.projects.iter().enumerate() {
            self.sidebar_items.push(SidebarItem::Project(pi));

            if self.expanded_projects.contains(&pi) {
                for (ci, collection) in project.collections.iter().enumerate() {
                    self.sidebar_items.push(SidebarItem::Collection(pi, ci));

                    if self.expanded_collections.contains(&(pi, ci)) {
                        for ri in 0..collection.requests.len() {
                            self.sidebar_items.push(SidebarItem::Request(pi, ci, ri));
                        }
                    }
                }
            }
        }
    }

    pub fn toggle_project_expand(&mut self, project_idx: usize) {
        if let Some(pos) = self.expanded_projects.iter().position(|&x| x == project_idx) {
            self.expanded_projects.remove(pos);
            self.expanded_collections.retain(|(p, _)| *p != project_idx);
        } else {
            self.expanded_projects.push(project_idx);
        }
        self.rebuild_sidebar_items();
    }

    pub fn toggle_collection_expand(&mut self, project_idx: usize, collection_idx: usize) {
        let key = (project_idx, collection_idx);
        if let Some(pos) = self.expanded_collections.iter().position(|&x| x == key) {
            self.expanded_collections.remove(pos);
        } else {
            self.expanded_collections.push(key);
        }
        self.rebuild_sidebar_items();
    }

    pub fn select_request(&mut self, project_idx: usize, collection_idx: usize, request_idx: usize) {
        if let Some(project) = self.workspace.projects.get(project_idx) {
            if let Some(collection) = project.collections.get(collection_idx) {
                if let Some(request) = collection.requests.get(request_idx) {
                    self.current_request = Some(request.clone());
                    self.current_request_path = Some((project_idx, collection_idx, request_idx));
                    self.url_input = request.url.clone();
                    self.body_input = request.body.clone();
                    self.headers_list = request.headers.iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect();
                    self.response = None;
                    self.error_message = None;
                }
            }
        }
    }

    pub fn save_current_request(&mut self) {
        if let (Some(request), Some((pi, ci, ri))) = (&mut self.current_request, self.current_request_path) {
            request.url = self.url_input.clone();
            request.body = self.body_input.clone();
            request.headers = self.headers_list.iter()
                .cloned()
                .collect();

            if let Some(project) = self.workspace.projects.get_mut(pi) {
                if let Some(collection) = project.collections.get_mut(ci) {
                    if let Some(stored_request) = collection.requests.get_mut(ri) {
                        *stored_request = request.clone();
                        self.needs_save = true;
                    }
                }
            }
        }
    }

    pub fn cycle_method(&mut self) {
        if let Some(ref mut request) = self.current_request {
            request.method = request.method.next();
            self.needs_save = true;
        }
    }

    pub fn add_project(&mut self, name: String) {
        let project = Project::new(name);
        self.workspace.projects.push(project);
        self.needs_save = true;
        self.rebuild_sidebar_items();
    }

    pub fn add_collection(&mut self, project_idx: usize, name: String) {
        if let Some(project) = self.workspace.projects.get_mut(project_idx) {
            let collection = Collection::new(name);
            project.collections.push(collection);
            self.needs_save = true;
            if !self.expanded_projects.contains(&project_idx) {
                self.expanded_projects.push(project_idx);
            }
            self.rebuild_sidebar_items();
        }
    }

    pub fn add_request(&mut self, project_idx: usize, collection_idx: usize, name: String) {
        if let Some(project) = self.workspace.projects.get_mut(project_idx) {
            if let Some(collection) = project.collections.get_mut(collection_idx) {
                let request = Request::new(name);
                collection.requests.push(request);
                self.needs_save = true;
                if !self.expanded_collections.contains(&(project_idx, collection_idx)) {
                    self.expanded_collections.push((project_idx, collection_idx));
                }
                self.rebuild_sidebar_items();
            }
        }
    }

    pub fn delete_selected(&mut self) {
        if self.sidebar_items.is_empty() {
            return;
        }

        let item = self.sidebar_items[self.sidebar_selected];
        match item {
            SidebarItem::Project(pi) => {
                self.workspace.projects.remove(pi);
                self.expanded_projects.retain(|&x| x != pi);
                self.expanded_collections.retain(|(p, _)| *p != pi);
                if self.current_request_path.map(|(p, _, _)| p) == Some(pi) {
                    self.current_request = None;
                    self.current_request_path = None;
                }
            }
            SidebarItem::Collection(pi, ci) => {
                if let Some(project) = self.workspace.projects.get_mut(pi) {
                    project.collections.remove(ci);
                    self.expanded_collections.retain(|&(p, c)| !(p == pi && c == ci));
                    if self.current_request_path.map(|(p, c, _)| (p, c)) == Some((pi, ci)) {
                        self.current_request = None;
                        self.current_request_path = None;
                    }
                }
            }
            SidebarItem::Request(pi, ci, ri) => {
                if let Some(project) = self.workspace.projects.get_mut(pi) {
                    if let Some(collection) = project.collections.get_mut(ci) {
                        collection.requests.remove(ri);
                        if self.current_request_path == Some((pi, ci, ri)) {
                            self.current_request = None;
                            self.current_request_path = None;
                        }
                    }
                }
            }
        }

        self.needs_save = true;
        self.rebuild_sidebar_items();

        if self.sidebar_selected >= self.sidebar_items.len() && !self.sidebar_items.is_empty() {
            self.sidebar_selected = self.sidebar_items.len() - 1;
        }
    }

    pub fn get_selected_project_index(&self) -> Option<usize> {
        if self.sidebar_items.is_empty() {
            return None;
        }
        match self.sidebar_items.get(self.sidebar_selected)? {
            SidebarItem::Project(pi) => Some(*pi),
            SidebarItem::Collection(pi, _) => Some(*pi),
            SidebarItem::Request(pi, _, _) => Some(*pi),
        }
    }

    pub fn get_selected_collection_index(&self) -> Option<(usize, usize)> {
        if self.sidebar_items.is_empty() {
            return None;
        }
        match self.sidebar_items.get(self.sidebar_selected)? {
            SidebarItem::Project(_) => None,
            SidebarItem::Collection(pi, ci) => Some((*pi, *ci)),
            SidebarItem::Request(pi, ci, _) => Some((*pi, *ci)),
        }
    }

    pub fn add_header(&mut self) {
        if !self.header_key_input.is_empty() {
            self.headers_list.push((
                self.header_key_input.clone(),
                self.header_value_input.clone(),
            ));
            self.header_key_input.clear();
            self.header_value_input.clear();
            self.needs_save = true;
        }
    }

    pub fn remove_header(&mut self, index: usize) {
        if index < self.headers_list.len() {
            self.headers_list.remove(index);
            self.needs_save = true;
        }
    }
}
