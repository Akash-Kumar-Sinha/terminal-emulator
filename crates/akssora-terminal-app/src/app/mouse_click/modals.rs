use winit::event_loop::ActiveEventLoop;

use crate::app::App;

impl App {
    pub(crate) fn handle_right_click(&mut self, mx: f32, my: f32, top_h: f32, sb_w: f32) -> bool {
        if self.explorer.is_expanded
            && mx >= sb_w
            && mx <= sb_w + self.explorer.width
            && my >= top_h
        {
            let target = self.explorer.hit_test_at(mx, my, top_h, sb_w);
            let path = match target {
                Some(akssora_terminal_ui::ExplorerTarget::RowItem { path, .. }) => Some(path),
                Some(akssora_terminal_ui::ExplorerTarget::RowToggle(path)) => Some(path),
                _ => None,
            };
            self.explorer.context_menu =
                Some(akssora_terminal_ui::ExplorerContextMenu::new(mx, my, path));
            if let Some(win) = self.window.as_ref() {
                win.request_redraw();
            }
            return true;
        }

        if let Some(menu) = &self.context_menu {
            if menu.contains(mx, my) {
                return true;
            }
            self.context_menu = None;
        }

        if self.pane_at(mx, my).is_some() {
            self.focus_pane_at(mx, my);
            self.open_context_menu(mx, my);
            return true;
        }

        false
    }

    pub(crate) fn handle_modal_and_overlay_clicks(
        &mut self,
        event_loop: &ActiveEventLoop,
        mx: f32,
        my: f32,
        w: f32,
        h: f32,
        top_h: f32,
    ) -> bool {
        // INFO: Left-click inside or outside open quick search palette
        if self.quick_search.is_open {
            let (qox, qoy, qow, qoh) = akssora_terminal_ui::QuickSearchUi::overlay_rect(
                w,
                top_h,
                self.quick_search.results.len(),
            );
            if mx >= qox && mx <= qox + qow && my >= qoy && my <= qoy + qoh {
                if let Some(idx) = self.quick_search.hit_test_item(mx, my, w, top_h)
                    && let Some(item) = self.quick_search.results.get(idx).cloned()
                {
                    self.quick_search.close();
                    self.open_editor_file(&item.path);
                }
            } else {
                self.quick_search.close();
            }
            if let Some(win) = self.window.as_ref() {
                win.request_redraw();
            }
            return true;
        }

        // Left-click on explorer context menu
        if let Some(menu) = &self.explorer.context_menu {
            if let Some(action) = menu.hit_test(mx, my) {
                let target_path = menu.target_path.clone();
                match action {
                    akssora_terminal_ui::ExplorerContextAction::OpenInEditor => {
                        if let Some(p) = target_path {
                            self.open_editor_file(&p);
                        }
                    }
                    akssora_terminal_ui::ExplorerContextAction::OpenInSplit => {
                        if let Some(p) = target_path {
                            self.open_file_in_split_pane(&p);
                        }
                    }
                    akssora_terminal_ui::ExplorerContextAction::OpenTerminalHere => {
                        let target_dir = target_path
                            .map(|p| {
                                if p.is_dir() {
                                    p
                                } else {
                                    p.parent()
                                        .unwrap_or(&self.explorer.tree.root_path)
                                        .to_path_buf()
                                }
                            })
                            .unwrap_or_else(|| self.explorer.tree.root_path.clone());
                        self.open_terminal_tab_in(Some(&target_dir));
                    }
                    akssora_terminal_ui::ExplorerContextAction::NewFile => {
                        let parent =
                            target_path.unwrap_or_else(|| self.explorer.tree.root_path.clone());
                        self.explorer.inline_input = Some(akssora_terminal_ui::InlineInputState {
                            mode: akssora_terminal_ui::InlineInputMode::NewFile,
                            text: String::new(),
                            parent_dir: parent,
                            error: None,
                        });
                    }
                    akssora_terminal_ui::ExplorerContextAction::NewFolder => {
                        let parent =
                            target_path.unwrap_or_else(|| self.explorer.tree.root_path.clone());
                        self.explorer.inline_input = Some(akssora_terminal_ui::InlineInputState {
                            mode: akssora_terminal_ui::InlineInputMode::NewFolder,
                            text: String::new(),
                            parent_dir: parent,
                            error: None,
                        });
                    }
                    akssora_terminal_ui::ExplorerContextAction::Rename => {
                        if let Some(p) = target_path {
                            let name = p
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_default();
                            self.explorer.inline_input =
                                Some(akssora_terminal_ui::InlineInputState {
                                    mode: akssora_terminal_ui::InlineInputMode::Rename {
                                        target: p.clone(),
                                    },
                                    text: name,
                                    parent_dir: p
                                        .parent()
                                        .unwrap_or(&self.explorer.tree.root_path)
                                        .to_path_buf(),
                                    error: None,
                                });
                        }
                    }
                    akssora_terminal_ui::ExplorerContextAction::Copy => {
                        if let Some(p) = target_path {
                            let parent = p
                                .parent()
                                .unwrap_or(&self.explorer.tree.root_path)
                                .to_path_buf();
                            let _ = self.explorer.tree.copy_entry(&p, &parent);
                        }
                    }
                    akssora_terminal_ui::ExplorerContextAction::CopyPath => {
                        if let Some(p) = target_path {
                            akssora_terminal_input::set_clipboard_text(&p.to_string_lossy());
                        }
                    }
                    akssora_terminal_ui::ExplorerContextAction::RevealInFileManager => {
                        if let Some(p) = target_path {
                            let reveal_dir = if p.is_dir() {
                                p
                            } else {
                                p.parent()
                                    .unwrap_or(&self.explorer.tree.root_path)
                                    .to_path_buf()
                            };
                            let _ = std::process::Command::new("xdg-open")
                                .arg(reveal_dir)
                                .spawn();
                        }
                    }
                    akssora_terminal_ui::ExplorerContextAction::Delete => {
                        if let Some(p) = target_path {
                            self.explorer.confirm_delete = Some(vec![p]);
                        }
                    }
                    akssora_terminal_ui::ExplorerContextAction::ToggleHidden => {
                        self.explorer.tree.toggle_hidden();
                    }
                }
                self.explorer.context_menu = None;
                if let Some(win) = self.window.as_ref() {
                    win.request_redraw();
                }
                return true;
            } else {
                self.explorer.context_menu = None;
            }
        }

        // Left-click on delete confirmation modal
        if self.explorer.confirm_delete.is_some() {
            if let Some(target) =
                akssora_terminal_ui::ExplorerUi::hit_test_confirm_dialog(mx, my, w, h)
            {
                match target {
                    akssora_terminal_ui::ExplorerTarget::ConfirmDeleteOk => {
                        let deleted = self.explorer.tree.delete_selected().unwrap_or_default();
                        for p in deleted {
                            self.handle_file_deleted(&p);
                        }
                        self.explorer.confirm_delete = None;
                    }
                    akssora_terminal_ui::ExplorerTarget::ConfirmDeleteCancel => {
                        self.explorer.confirm_delete = None;
                    }
                    _ => {}
                }
            } else {
                self.explorer.confirm_delete = None;
            }
            if let Some(win) = self.window.as_ref() {
                win.request_redraw();
            }
            return true;
        }

        // Left-click on terminal context menu
        if let Some(menu) = &self.context_menu {
            if let Some(action) = menu.hit_test(mx, my) {
                self.handle_context_menu_action(action);
                return true;
            } else {
                self.context_menu = None;
                if let Some(win) = self.window.as_ref() {
                    win.request_redraw();
                }
            }
        }

        // 0. Unsaved changes modal click interaction
        if self.unsaved_prompt_target.is_some() {
            let modal_w = 480.0f32.min(w - 40.0);
            let modal_h = 160.0f32;
            let mx_box = (w - modal_w) / 2.0;
            let my_box = (h - modal_h) / 2.0;
            let btn_y = my_box + modal_h - 48.0;
            let btn_h = 32.0;
            let btn_gap = 12.0;

            let is_all = matches!(
                self.unsaved_prompt_target,
                Some(crate::app::UnsavedCloseAction::CloseAll)
            );
            let save_w = if is_all { 145.0 } else { 125.0 };
            let dont_w = 135.0;
            let cancel_w = 125.0;
            let total_buttons_w = save_w + dont_w + cancel_w + (btn_gap * 2.0);
            let start_btn_x = mx_box + (modal_w - total_buttons_w) / 2.0;

            let save_x = start_btn_x;
            let dont_x = save_x + save_w + btn_gap;
            let cancel_x = dont_x + dont_w + btn_gap;

            if mx >= save_x && mx < save_x + save_w && my >= btn_y && my < btn_y + btn_h {
                self.save_prompt_action();
                return true;
            } else if mx >= dont_x && mx < dont_x + dont_w && my >= btn_y && my < btn_y + btn_h {
                self.discard_prompt_action();
                return true;
            } else if (mx >= cancel_x
                && mx < cancel_x + cancel_w
                && my >= btn_y
                && my < btn_y + btn_h)
                || mx < mx_box
                || mx >= mx_box + modal_w
                || my < my_box
                || my >= my_box + modal_h
            {
                self.cancel_prompt_action();
                return true;
            }
            return true;
        }

        // 1. Exit confirmation modal click interaction
        if self.prompt_confirm_exit {
            let modal_w = 460.0f32.min(w - 40.0);
            let modal_h = 160.0f32;
            let mx_box = (w - modal_w) / 2.0;
            let my_box = (h - modal_h) / 2.0;
            let btn_y = my_box + modal_h - 48.0;
            let btn_h = 32.0;
            let btn_gap = 16.0;
            let c_w = 160.0;
            let can_w = 140.0;
            let total_buttons_w = c_w + can_w + btn_gap;
            let start_btn_x = mx_box + (modal_w - total_buttons_w) / 2.0;
            let c_x = start_btn_x;
            let can_x = c_x + c_w + btn_gap;

            if mx >= c_x && mx < c_x + c_w && my >= btn_y && my < btn_y + btn_h {
                self.close_all_tabs();
                event_loop.exit();
                return true;
            } else if (mx >= can_x && mx < can_x + can_w && my >= btn_y && my < btn_y + btn_h)
                || mx < mx_box
                || mx >= mx_box + modal_w
                || my < my_box
                || my >= my_box + modal_h
            {
                self.prompt_confirm_exit = false;
                if let Some(win) = self.window.as_ref() {
                    win.request_redraw();
                }
                return true;
            }
            return true;
        }

        false
    }
}
