use crate::{
    actions::{self},
    app::App,
    ascii,
    config::{self, Mode, Setting},
    error::AppError,
    leaderboard::{Leaderboard, LeaderboardMotion, SortColumn},
    log_warn,
    menu::{MenuContext, MenuMotion},
    modal::{Modal, ModalContext},
    notify_info, theme,
    variants::{CursorVariant, PickerVariant, ResultsVariant},
};

#[derive(Debug, Clone, Copy)]
pub struct AppHandler;

impl AppHandler {
    pub fn handle_input(self, app: &mut App, chr: char) -> Result<(), AppError> {
        if app.tracker.is_complete() {
            return Ok(());
        }
        match app.tracker.type_char(chr) {
            Ok(()) => Ok(()),
            Err(AppError::IllegalSpaceCharacter) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub fn handle_change_theme(self, _app: &mut App, theme_name: String) -> Result<(), AppError> {
        theme::set_as_current_theme(&theme_name)?;
        notify_info!(format!("Theme change: {theme_name}"));
        Ok(())
    }

    pub fn handle_randomize_theme(self, _app: &mut App) -> Result<(), AppError> {
        theme::use_random_theme()?;
        notify_info!(format!("Theme change: {}", theme::current_theme().id));
        Ok(())
    }

    pub fn handle_toggle_setting(self, app: &mut App, setting: Setting) -> Result<(), AppError> {
        app.config.toggle(&setting)?;
        if setting.should_trigger_restart() {
            app.restart()?;
        }
        Ok(())
    }

    pub fn handle_enable_setting(self, app: &mut App, setting: Setting) -> Result<(), AppError> {
        if !app.config.is_enabled(setting.clone()) {
            app.config.toggle(&setting)?;
            if setting.should_trigger_restart() {
                app.restart()?;
            }
        }
        Ok(())
    }

    pub fn handle_disable_setting(self, app: &mut App, setting: Setting) -> Result<(), AppError> {
        if app.config.is_enabled(setting.clone()) {
            app.config.toggle(&setting)?;
            if setting.should_trigger_restart() {
                app.restart()?;
            }
        }
        Ok(())
    }

    pub fn handle_command_palette_toggle(self, app: &mut App) -> Result<(), AppError> {
        if app.menu.is_open() {
            // if the cmd palette is currently open, then close it
            if let Some(current_menu) = app.menu.current_menu() {
                if current_menu.is_cmd_palette {
                    return AppHandler.handle_menu_close(app);
                }
            }
            // we don't want cmd palette and the actual menu to be open at the same time,
            // thus if the menu is currently opened, then close it
            AppHandler.handle_menu_close(app)?;
        }
        AppHandler.handle_menu_open(app, MenuContext::CommandPalette)
    }

    pub fn handle_menu_open(self, app: &mut App, ctx: MenuContext) -> Result<(), AppError> {
        app.menu.open(ctx, &app.config)?;
        app.try_preview()?;
        app.tracker.toggle_pause();
        Ok(())
    }

    pub fn handle_menu_close(self, app: &mut App) -> Result<(), AppError> {
        // TODO: this clearing of preview should be done cleanly
        app.restore_cursor_style();
        app.menu.close()?;
        app.tracker.toggle_pause();
        Ok(())
    }

    pub fn handle_menu_toggle(self, app: &mut App) -> Result<(), AppError> {
        if app.menu.is_open() {
            return AppHandler.handle_menu_close(app);
        }
        AppHandler.handle_menu_open(app, MenuContext::Root)
    }

    pub fn handle_menu_backtrack(self, app: &mut App) -> Result<(), AppError> {
        // TODO: this clearing of preview should be done cleanly
        theme::cancel_theme_preview();
        app.menu.back()?;
        if !app.menu.is_open() {
            app.restore_cursor_style();
            app.tracker.toggle_pause();
        }
        Ok(())
    }

    pub fn handle_menu_navigate(self, app: &mut App, motion: MenuMotion) -> Result<(), AppError> {
        app.menu.navigate(motion);
        app.try_preview()?;
        Ok(())
    }

    pub fn handle_menu_shortcut(self, app: &mut App, shortcut: char) -> Result<(), AppError> {
        if let Some(menu) = app.menu.current_menu_mut() {
            if let Some((idx, _)) = menu.find_by_shortcut(shortcut) {
                menu.set_current_index(idx);
                return AppHandler.handle_menu_select(app);
            }
        }
        Ok(())
    }

    pub fn handle_menu_select(self, app: &mut App) -> Result<(), AppError> {
        if let Ok(Some(action)) = app.menu.select(&app.config) {
            actions::handle_action(app, action)?;
            // note: the action above could've been a menu closing action.
            if !app.menu.is_open() {
                theme::cancel_theme_preview();
                app.restore_cursor_style();
                app.tracker.toggle_pause();
            }
        }
        Ok(())
    }

    pub fn handle_menu_exit_search(self, app: &mut App) -> Result<(), AppError> {
        // if we are in command palette we should close the whole menu
        if let Some(menu) = app.menu.current_menu() {
            if menu.is_cmd_palette {
                return AppHandler.handle_menu_close(app);
            }
        }
        // we are in the normal menu, just exit search and enter normal mode
        app.menu.exit_search();
        Ok(())
    }

    pub fn handle_menu_backspace_search(self, app: &mut App) -> Result<(), AppError> {
        app.menu.backspace_search();
        // app.try_preview()?;
        Ok(())
    }

    pub fn handle_menu_init_search(self, app: &mut App) -> Result<(), AppError> {
        app.menu.init_search();
        Ok(())
    }

    pub fn handle_menu_update_search(self, app: &mut App, query: String) -> Result<(), AppError> {
        let current_query = app.menu.search_query().to_string();
        let new_query = if query.is_empty() {
            String::new()
        } else {
            format!("{}{}", current_query, query)
        };
        app.menu.update_search(new_query);
        app.try_preview()?;

        Ok(())
    }

    pub fn handle_modal_open(self, app: &mut App, ctx: ModalContext) -> Result<(), AppError> {
        app.modal = Some(Modal::new(ctx));
        Ok(())
    }

    pub fn handle_modal_close(self, app: &mut App) -> Result<(), AppError> {
        app.modal = None;
        Ok(())
    }

    pub fn handle_modal_backspace(self, app: &mut App) -> Result<(), AppError> {
        if let Some(modal) = app.modal.as_mut() {
            modal.handle_backspace();
        }
        Ok(())
    }

    pub fn handle_modal_input(self, app: &mut App, chr: char) -> Result<(), AppError> {
        if let Some(modal) = app.modal.as_mut() {
            modal.handle_input(chr);
        }
        Ok(())
    }

    pub fn handle_modal_confirm(self, app: &mut App) -> Result<(), AppError> {
        if let Some(modal) = app.modal.as_mut() {
            // NOTE(ema): this would've been so clean, but unfortunately we don't know wich context
            // we currently at in `keymap_builder`. To what we would need to map `Action::ModalConfirm` to?
            // Maybe in the future i've grinded enough intellect xp to be able to tackle this
            // actions::handle_action(app, action);

            match modal.ctx {
                ModalContext::CustomTime => {
                    // TODO: find a cleaner way of doing this. Maybe have get_value handle the parsing inside?
                    if let Ok(val) = modal.get_value() {
                        if let Ok(secs) = val.parse::<usize>() {
                            app.config.change_mode(Mode::with_time(secs))?;
                            app.restart()?
                        }
                    }
                }
                ModalContext::CustomWordCount => {
                    // TODO: find a cleaner way of doing this. Maybe have get_value handle the parsing inside?
                    if let Ok(val) = modal.get_value() {
                        if let Ok(count) = val.parse::<usize>() {
                            app.config.change_mode(Mode::with_words(count))?;
                            app.restart()?
                        }
                    }
                }
                ModalContext::CustomLineCount => {
                    // TODO: find a cleaner way of doing this. Maybe have get_value handle the parsing inside?
                    if let Ok(val) = modal.get_value() {
                        if let Ok(count) = val.parse::<u8>() {
                            app.config.change_visible_lines_count(count);
                            app.restart()?
                        }
                    }
                }
                ModalContext::ExitConfirmation => app.quit()?,
            }
        }
        AppHandler.handle_modal_close(app)?;
        AppHandler.handle_menu_close(app)?;
        Ok(())
    }

    pub fn handle_backspace(self, app: &mut App) -> Result<(), AppError> {
        match app.tracker.backspace() {
            Ok(()) => Ok(()),
            Err(AppError::TypingTestNotInProgress) => Ok(()),
            Err(AppError::IllegalBackspace) => Ok(()),
            Err(AppError::IllegalSpaceCharacter) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub fn handle_set_line_count(self, app: &mut App, line_count: u8) -> Result<(), AppError> {
        app.config.change_visible_lines_count(line_count);
        Ok(())
    }

    pub fn handle_set_cursor(self, app: &mut App, variant: CursorVariant) -> Result<(), AppError> {
        app.config.change_cursor_variant(variant);
        // app.restart()?;
        Ok(())
    }

    pub fn handle_set_picker(self, app: &mut App, variant: PickerVariant) -> Result<(), AppError> {
        app.config.change_picker_variant(variant);
        Ok(())
    }

    pub fn handle_set_result(self, app: &mut App, variant: ResultsVariant) -> Result<(), AppError> {
        app.config.change_results_variant(variant);
        Ok(())
    }

    pub fn handle_set_time(self, app: &mut App, secs: usize) -> Result<(), AppError> {
        app.config.change_mode(config::Mode::with_time(secs))?;
        app.restart()?;
        Ok(())
    }

    pub fn handle_set_words(self, app: &mut App, count: usize) -> Result<(), AppError> {
        app.config.change_mode(config::Mode::with_words(count))?;
        app.restart()?;
        Ok(())
    }

    pub fn handle_set_language(self, app: &mut App, lang: String) -> Result<(), AppError> {
        app.config.change_language(lang);
        app.restart()?;
        Ok(())
    }

    pub fn handle_set_ascii_art(self, app: &mut App, art: String) -> Result<(), AppError> {
        // NOTE(ema): this feels a little bit to "side-effecty", but selecting an ascii art without
        // having the `ResultsVariant::Neofetch` as the current variant feels pointless, so yeah.
        if app.config.current_results_variant() != ResultsVariant::Neofetch {
            app.config.change_results_variant(ResultsVariant::Neofetch);
        }
        app.config.change_ascii_art(art.clone());
        notify_info!(format!("Art change: {art}"));
        Ok(())
    }

    // TODO: refactor this two `handle_cycle` functions into a single one that receives
    // either `Direction::Next` or `Direction::Prev`
    pub fn handle_cycle_next_art(self, app: &mut App) -> Result<(), AppError> {
        let current = app.config.current_ascii_art();
        let list = ascii::list_ascii();
        if let Some(idx) = list.iter().position(|a| a == &current) {
            let next_idx = (idx + 1) % list.len();
            let next_art = list[next_idx].clone();
            app.config.change_ascii_art(next_art);
        }
        Ok(())
    }

    pub fn handle_cycle_prev_art(self, app: &mut App) -> Result<(), AppError> {
        let current = app.config.current_ascii_art();
        let list = ascii::list_ascii();
        if let Some(idx) = list.iter().position(|a| a == &current) {
            let prev_idx = if idx == 0 { list.len() - 1 } else { idx - 1 };
            let prev_art = list[prev_idx].clone();
            app.config.change_ascii_art(prev_art);
        }
        Ok(())
    }

    pub fn handle_leaderboard_open(self, app: &mut App) -> Result<(), AppError> {
        let Some(ref db) = app.db else {
            log_warn!("Tried opening the leaderboard without a db instance");
            return Ok(());
        };

        if app.leaderboard.is_none() {
            app.leaderboard = Some(Leaderboard::new());
        }

        if let Some(ref mut leaderboard) = app.leaderboard {
            leaderboard.open(db);
        }

        Ok(())
    }

    pub fn handle_leaderboard_close(self, app: &mut App) -> Result<(), AppError> {
        if let Some(ref mut leaderboard) = app.leaderboard {
            leaderboard.close();
            app.leaderboard = None;
        }
        Ok(())
    }

    pub fn handle_leaderboard_toggle(self, app: &mut App) -> Result<(), AppError> {
        let Some(ref db) = app.db else {
            log_warn!("Tried toggling the leaderboard without a db instance");
            return Ok(());
        };

        // don't be opening wild leaderboard overlays if we have other overlays alreay opened.
        if app.modal.is_some() || app.menu.is_open() {
            return Ok(());
        }

        // TODO: i don't like this pattern, feels wrong but works for now. Refactor me later.
        if app.leaderboard.is_some() {
            app.leaderboard = None;
            app.tracker.unpause();
        } else if app.leaderboard.is_none() {
            app.leaderboard = Some(Leaderboard::new());
            app.tracker.pause();
        }

        if let Some(ref mut leaderboard) = app.leaderboard {
            leaderboard.toggle(db);
        }

        Ok(())
    }

    pub fn handle_leaderboard_sort(self, app: &mut App, col: SortColumn) -> Result<(), AppError> {
        let Some(ref db) = app.db else {
            return Ok(());
        };

        if let Some(ref mut leaderboard) = app.leaderboard {
            leaderboard.sort(col, db);
        }

        Ok(())
    }

    pub fn handle_leaderboard_nav(
        self,
        app: &mut App,
        motion: LeaderboardMotion,
    ) -> Result<(), AppError> {
        let Some(ref db) = app.db else {
            return Ok(());
        };

        if let Some(ref mut leaderboard) = app.leaderboard {
            leaderboard.navigate(db, motion);
        }

        Ok(())
    }
}
