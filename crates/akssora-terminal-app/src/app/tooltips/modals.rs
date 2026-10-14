use crate::app::App;
use akssora_terminal_ui::Tooltip;

// INFO: Tooltip resolution for modal confirmation dialogs (exit confirmation, unsaved changes).
pub(crate) fn check_modal_tooltip(
    app: &App,
    mx: f32,
    my: f32,
    window_w: f32,
    window_h: f32,
) -> Option<Tooltip> {
    if app.prompt_confirm_exit {
        let modal_w = 460.0f32.min(window_w - 40.0);
        let modal_h = 160.0f32;
        let mx_box = (window_w - modal_w) / 2.0;
        let my_box = (window_h - modal_h) / 2.0;
        let btn_y = my_box + modal_h - 48.0;
        let btn_h = 32.0;
        let btn_gap = 16.0;
        let c_w = 160.0;
        let can_w = 140.0;
        let total_buttons_w = c_w + can_w + btn_gap;
        let start_btn_x = mx_box + (modal_w - total_buttons_w) / 2.0;

        if mx >= start_btn_x && mx <= start_btn_x + c_w && my >= btn_y && my <= btn_y + btn_h {
            return Some(Tooltip::for_target(
                "Exit Application",
                Some("Enter".to_string()),
                (start_btn_x, btn_y, c_w, btn_h),
            ));
        } else if mx >= start_btn_x + c_w + btn_gap
            && mx <= start_btn_x + total_buttons_w
            && my >= btn_y
            && my <= btn_y + btn_h
        {
            return Some(Tooltip::for_target(
                "Cancel and Return",
                Some("Esc".to_string()),
                (start_btn_x + c_w + btn_gap, btn_y, can_w, btn_h),
            ));
        }
    } else if app.unsaved_prompt_target.is_some() {
        let modal_w = 480.0f32.min(window_w - 40.0);
        let modal_h = 160.0f32;
        let mx_box = (window_w - modal_w) / 2.0;
        let my_box = (window_h - modal_h) / 2.0;
        let btn_y = my_box + modal_h - 48.0;
        let btn_h = 32.0;
        let btn_gap = 12.0;
        let save_w = 140.0;
        let dont_w = 150.0;
        let cancel_w = 130.0;
        let total_buttons_w = save_w + dont_w + cancel_w + (btn_gap * 2.0);
        let start_btn_x = mx_box + (modal_w - total_buttons_w) / 2.0;

        let s_x = start_btn_x;
        let d_x = s_x + save_w + btn_gap;
        let c_x = d_x + dont_w + btn_gap;

        if mx >= s_x && mx <= s_x + save_w && my >= btn_y && my <= btn_y + btn_h {
            return Some(Tooltip::for_target(
                "Save Changes",
                Some("Enter".to_string()),
                (s_x, btn_y, save_w, btn_h),
            ));
        } else if mx >= d_x && mx <= d_x + dont_w && my >= btn_y && my <= btn_y + btn_h {
            return Some(Tooltip::for_target(
                "Discard Changes",
                Some("D".to_string()),
                (d_x, btn_y, dont_w, btn_h),
            ));
        } else if mx >= c_x && mx <= c_x + cancel_w && my >= btn_y && my <= btn_y + btn_h {
            return Some(Tooltip::for_target(
                "Cancel Operation",
                Some("Esc".to_string()),
                (c_x, btn_y, cancel_w, btn_h),
            ));
        }
    }

    None
}
