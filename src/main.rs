use macroquad::audio::{
    PlaySoundParams, Sound, load_sound, play_sound, set_sound_volume, stop_sound,
};
use macroquad::prelude::*;
use macroquad::ui::{Skin, root_ui};

// (Filename, Accurate Duration in Seconds)
const TRACKS: &[(&str, f32)] = &[("Safe.flac", 83.0), ("hideandseek.flac", 46.0)];

#[macroquad::main("vyrtu_player")]
async fn main() {
    // 1. Bake Font (Must be in project root alongside Cargo.toml)
    let font_data = include_bytes!("../JetBrainsMonoNerdFont-Regular.ttf");
    let jbm_font = load_ttf_font_from_bytes(font_data).expect("Font failed");

    // 2. Custom UI Skinning
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

    // Progress and Timing logic
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

        // --- VISUALISER (Simulated Spectrum) ---
        if playing {
            let bars = 24;
            let spacing = 8.0;
            let bar_w = 12.0;
            let total_w = (bars as f32 * bar_w) + ((bars - 1) as f32 * spacing);
            let start_x = (screen_width() - total_w) / 2.0;
            let base_y = screen_height() / 2.0 + 40.0;

            for i in 0..bars {
                let speed = 6.0 + (i as f64 * 0.6); // Higher index = faster flicker
                let t = get_time() * speed + (i as f64 * 0.4);
                let wave = (t.sin() as f32 + 1.0) * 0.5;
                let bar_h = wave * (60.0 + i as f32 * 1.5) * volume + 4.0;
                draw_rectangle(
                    start_x + i as f32 * (bar_w + spacing),
                    base_y - bar_h,
                    bar_w,
                    bar_h,
                    Color::from_rgba(100, 100, 255, 180),
                );
            }
        }

        // --- BOTTOM BAR ---
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
                        status_message = format!("   FINISHED: {}", TRACKS[current_track_index].0);
                    }
                }
            }

            // Progress Bar
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

            // Timer Display
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

            // PLAY/STOP Control
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
                    // Resume timer from where it was
                    start_time = get_time() - (elapsed_time as f64);
                    status_message = format!("   PLAYING: {}", TRACKS[current_track_index].0);
                } else {
                    stop_sound(sound);
                    playing = false;
                    elapsed_time = 0.0; // Reset progress bar on Stop
                    status_message = format!("   STOPPED: {}", TRACKS[current_track_index].0);
                }
            }

            // LOOP Toggle
            if root_ui().button(
                vec2(150.0, bar_y + 20.0),
                if is_looped { "   LOOP" } else { "   LOOP" },
            ) {
                is_looped = !is_looped;
                if playing {
                    stop_sound(sound);
                    play_sound(
                        sound,
                        PlaySoundParams {
                            looped: is_looped,
                            volume,
                        },
                    );
                    start_time = get_time() - (elapsed_time as f64);
                }
            }

            // Volume Section
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
