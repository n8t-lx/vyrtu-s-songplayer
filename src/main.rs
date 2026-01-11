use macroquad::audio::{
    PlaySoundParams, Sound, load_sound, play_sound, set_sound_volume, stop_sound,
};
use macroquad::prelude::*;
use macroquad::ui::{Skin, root_ui};

const TRACKS: &[(&str, f32)] = &[("Safe.flac", 83.0), ("hideandseek.flac", 46.0)];

#[macroquad::main("VYRTU_PLAYER")]
async fn main() {
    let font_data = include_bytes!("../jbm_slim.ttf");
    let jbm_font = load_ttf_font_from_bytes(font_data).expect("Slim font failed");

    let ui_skin = {
        let label_style = root_ui()
            .style_builder()
            .font(font_data)
            .unwrap()
            .font_size(20)
            .text_color(Color::from_rgba(200, 200, 200, 255))
            .build();
        let button_style = root_ui()
            .style_builder()
            .background_margin(RectOffset::new(5.0, 5.0, 5.0, 5.0))
            .margin(RectOffset::new(10.0, 10.0, 0.0, 0.0))
            .font(font_data)
            .unwrap()
            .font_size(22)
            .text_color(WHITE)
            .color(Color::from_rgba(40, 40, 40, 255))
            .color_hovered(Color::from_rgba(60, 60, 60, 255))
            .color_clicked(Color::from_rgba(100, 100, 100, 255))
            .build();
        Skin {
            label_style,
            button_style,
            ..root_ui().default_skin()
        }
    };
    root_ui().push_skin(&ui_skin);

    let mut playing = false;
    let mut is_looped = false;
    let mut volume = 0.6;
    let mut current_sound: Option<Sound> = None;
    let mut current_track_index = 0;
    let mut status_message = "   READY".to_string();
    let mut start_time = 0.0;
    let mut elapsed_time = 0.0;

    loop {
        clear_background(Color::from_rgba(10, 10, 10, 255));
        let text_params = TextParams {
            font: Some(&jbm_font),
            font_size: 26,
            color: WHITE,
            ..Default::default()
        };

        // TITLE IS BACK
        draw_text_ex("   VYRTU_PLAYER", 20.0, 40.0, text_params.clone());
        draw_text_ex(
            &status_message,
            20.0,
            70.0,
            TextParams {
                font_size: 16,
                color: GRAY,
                ..text_params.clone()
            },
        );

        for (index, (track_name, _)) in TRACKS.iter().enumerate() {
            let btn_y = 120.0 + (index as f32 * 45.0);
            if root_ui().button(vec2(20.0, btn_y), format!("   {}", track_name)) {
                if let Some(s) = &current_sound {
                    stop_sound(s);
                }
                playing = false;
                elapsed_time = 0.0;
                if let Ok(s) = load_sound(track_name).await {
                    current_sound = Some(s);
                    current_track_index = index;
                    status_message = format!("   READY: {}", track_name);
                }
            }
        }

        if playing {
            let bars = 18;
            let spacing = 10.0;
            let bar_w = 14.0;
            let start_x =
                (screen_width() - ((bars as f32 * bar_w) + ((bars - 1) as f32 * spacing))) / 2.0;
            let base_y = screen_height() / 2.0 + 40.0;
            for i in 0..bars {
                let t = get_time() * (5.0 + i as f64 * 0.8) + (i as f64 * 0.4);
                let bar_h = ((t.sin() as f32 + 1.0) * 0.5) * (50.0 + i as f32 * 2.0) * volume + 4.0;
                draw_rectangle(
                    start_x + i as f32 * (bar_w + spacing),
                    base_y - bar_h,
                    bar_w,
                    bar_h,
                    Color::from_rgba(100, 100, 255, 140),
                );
            }
        }

        let bar_y = screen_height() - 120.0;
        if let Some(sound) = &current_sound {
            let (_, duration) = TRACKS[current_track_index];
            if playing {
                elapsed_time = (get_time() - start_time) as f32;
                if elapsed_time >= duration {
                    if is_looped {
                        start_time = get_time();
                    } else {
                        playing = false;
                        elapsed_time = 0.0;
                        stop_sound(sound);
                    }
                }
            }
            draw_rectangle(
                20.0,
                bar_y - 20.0,
                screen_width() - 40.0,
                4.0,
                Color::from_rgba(30, 30, 30, 255),
            );
            draw_rectangle(
                20.0,
                bar_y - 20.0,
                (screen_width() - 40.0) * (elapsed_time / duration).clamp(0.0, 1.0),
                4.0,
                Color::from_rgba(100, 100, 255, 255),
            );

            if root_ui().button(vec2(20.0, bar_y + 20.0), if !playing { "  " } else { "  " }) {
                if !playing {
                    play_sound(
                        sound,
                        PlaySoundParams {
                            looped: is_looped,
                            volume,
                        },
                    );
                    playing = true;
                    start_time = get_time() - (elapsed_time as f64);
                } else {
                    stop_sound(sound);
                    playing = false;
                    elapsed_time = 0.0;
                }
            }
            if root_ui().button(
                vec2(70.0, bar_y + 20.0),
                if is_looped { "  " } else { "  " },
            ) {
                is_looped = !is_looped;
            }
            root_ui().slider(1, "", 0.0..1.0, &mut volume);
            set_sound_volume(sound, volume);
        }
        next_frame().await
    }
}
