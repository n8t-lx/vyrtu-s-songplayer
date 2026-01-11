use macroquad::audio::{
    PlaySoundParams, Sound, load_sound, play_sound, set_sound_volume, stop_sound,
};
use macroquad::prelude::*;
use macroquad::ui::{Skin, root_ui};

// UPDATE THESE: (Filename, Duration in Seconds)
const TRACKS: &[(&str, f32)] = &[("Safe.flac", 185.0), ("hideandseek.flac", 240.0)];

#[macroquad::main("vyrtu_player")]
async fn main() {
    let font_data = include_bytes!("../JetBrainsMonoNerdFont-Regular.ttf");
    let jbm_font = load_ttf_font_from_bytes(font_data).expect("Font failed");

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
    let mut status_message = "   SYSTEM READY".to_string();

    // Progress tracking
    let mut start_time = 0.0;
    let mut elapsed_time = 0.0;

    loop {
        clear_background(Color::from_rgba(15, 15, 15, 255));
        let text_params = TextParams {
            font: Some(&jbm_font),
            font_size: 26,
            color: WHITE,
            ..Default::default()
        };

        // --- HEADER ---
        draw_text_ex("   VYRTU_SONGPLAYER", 20.0, 40.0, text_params.clone());
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

        // --- LIBRARY ---
        draw_text_ex(
            "   LIBRARY",
            20.0,
            120.0,
            TextParams {
                font_size: 18,
                color: Color::from_rgba(100, 100, 255, 255),
                ..text_params.clone()
            },
        );
        for (index, (track_name, _)) in TRACKS.iter().enumerate() {
            let btn_y = 140.0 + (index as f32 * 45.0);
            let active_mark = if playing && current_track_index == index {
                "   "
            } else {
                "  "
            };

            if root_ui().button(vec2(20.0, btn_y), format!("{}{}", active_mark, track_name)) {
                if let Some(s) = &current_sound {
                    stop_sound(s);
                }
                playing = false;
                elapsed_time = 0.0;
                match load_sound(track_name).await {
                    Ok(s) => {
                        current_sound = Some(s);
                        current_track_index = index;
                        status_message = format!("   LOADED: {}", track_name);
                    }
                    Err(_) => status_message = format!("   FAILED: {}", track_name),
                }
            }
        }

        // --- BOTTOM BAR & PROGRESS ---
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
                        elapsed_time = duration;
                    }
                }
            }

            // Draw Bar
            let progress = (elapsed_time / duration).clamp(0.0, 1.0);
            draw_rectangle(
                20.0,
                bar_y - 20.0,
                screen_width() - 40.0,
                4.0,
                Color::from_rgba(40, 40, 40, 255),
            );
            draw_rectangle(
                20.0,
                bar_y - 20.0,
                (screen_width() - 40.0) * progress,
                4.0,
                Color::from_rgba(100, 100, 255, 255),
            );

            // Draw Timer
            let time_txt = format!(
                "{}:{:02} / {}:{:02}",
                (elapsed_time / 60.0) as i32,
                (elapsed_time % 60.0) as i32,
                (duration / 60.0) as i32,
                (duration % 60.0) as i32
            );
            draw_text_ex(
                &time_txt,
                20.0,
                bar_y + 10.0,
                TextParams {
                    font_size: 14,
                    color: GRAY,
                    ..text_params.clone()
                },
            );

            // Controls
            let play_label = if !playing { "   PLAY" } else { "   STOP" };
            if root_ui().button(vec2(20.0, bar_y + 20.0), play_label) {
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
                }
            }

            if root_ui().button(
                vec2(150.0, bar_y + 20.0),
                if is_looped { "   LOOP" } else { "   LOOP" },
            ) {
                is_looped = !is_looped;
            }

            let vol_x = screen_width() - 200.0;
            if vol_x > 300.0 {
                draw_text_ex(
                    &format!("   {:.0}%", volume * 100.0),
                    vol_x,
                    bar_y + 35.0,
                    TextParams {
                        font_size: 18,
                        ..text_params.clone()
                    },
                );
                root_ui().slider(1, "", 0.0..1.0, &mut volume);
                set_sound_volume(sound, volume);
            }
        }

        next_frame().await
    }
}
