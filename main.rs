#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

// use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::main;
use esp_hal::rmt::Rmt;
use esp_hal::time::{Duration, Instant};
use esp_println::println;

// Mengaktifkan Pull-Down Internal pada GPIO
use esp_hal::gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull};

use esp_hal_smartled::{SmartLedsAdapter, smart_led_buffer};
use smart_leds::{brightness, RGB8, SmartLedsWrite};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // Menghilangkan pesan pembuka agar terminal bersih hanya berisi data angka
    let delay = Delay::new();

    // 1. Inisialisasi LED RGB Bawaan di Pin 48
    let rmt = Rmt::new(peripherals.RMT, esp_hal::time::Rate::from_mhz(80)).unwrap();
    let rmt_channel = rmt.channel0;
    let mut rmt_buffer = smart_led_buffer!(1);
    let mut led_builtin = SmartLedsAdapter::new(rmt_channel, peripherals.GPIO48, &mut rmt_buffer);

    // 2. Inisialisasi LED Eksternal (GPIO 8: Hijau, GPIO 9: Merah, GPIO 10: Kuning)
    let mut ext_led_hijau = Output::new(peripherals.GPIO8, Level::Low, OutputConfig::default());
    let mut ext_led_merah = Output::new(peripherals.GPIO9, Level::Low, OutputConfig::default());
    let mut ext_led_kuning = Output::new(peripherals.GPIO10, Level::Low, OutputConfig::default());

    // 3. Inisialisasi Sensor 1 (GPIO 16 & 17) - Pull-Down Aktif
    let mut trigger1 = Output::new(peripherals.GPIO16, Level::Low, OutputConfig::default());
    let echo1 = Input::new(peripherals.GPIO17, InputConfig::default().with_pull(Pull::Down));

    // 4. Inisialisasi Sensor 2 (GPIO 6 & 7) - Pull-Down Aktif
    let mut trigger2 = Output::new(peripherals.GPIO6, Level::Low, OutputConfig::default());
    let echo2 = Input::new(peripherals.GPIO7, InputConfig::default().with_pull(Pull::Down));

    // 5. Inisialisasi Sensor 3 (GPIO 15 & 18) - Pull-Down Aktif
    let mut trigger3 = Output::new(peripherals.GPIO15, Level::Low, OutputConfig::default());
    let echo3 = Input::new(peripherals.GPIO18, InputConfig::default().with_pull(Pull::Down));

    // Variabel state machine untuk mencatat status sensor terakhir yang sehat
    let mut last_active_sensor = 1;

    loop {
        let mut dist1 = -1.0;
        let mut dist2 = -1.0;
        let mut dist3 = -1.0;

        let mut active_sensor = 0;
        let mut final_distance = -1.0;

        // ==========================================
        // TAHAP 1: COBA BACA SENSOR 1
        // ==========================================
        trigger1.set_high();
        delay.delay_micros(10);
        trigger1.set_low();

        let mut timeout = Instant::now();
        let mut s1_failed = false;
        while echo1.is_low() {
            if timeout.elapsed() > Duration::from_millis(40) {
                s1_failed = true;
                break;
            }
        }
        if !s1_failed {
            let echo_start1 = Instant::now();
            while echo1.is_high() {
                if echo_start1.elapsed() > Duration::from_millis(40) {
                    s1_failed = true;
                    break;
                }
            }
            if !s1_failed {
                dist1 = (echo_start1.elapsed().as_micros() as f32 * 0.0343) / 2.0;
            }
        }

        // Evaluasi Sensor 1 (Rentang normal 2 s.d 400cm)
        if dist1 >= 2.0 && dist1 <= 400.0 {
            active_sensor = 1;
            final_distance = dist1;
            last_active_sensor = 1; // Pulihkan status jika Sensor 1 sehat kembali
        } else {
            // ==========================================
            // TRANSISI: SENSOR 1 BARU SAJA MATI!
            // ==========================================
            if last_active_sensor == 1 {
                ext_led_hijau.set_low();
                ext_led_merah.set_low();

                // Berkedip 5 kali dalam 3 detik (300ms Nyala, 300ms Mati)
                for _ in 0..5 {
                    ext_led_kuning.set_high();
                    delay.delay_millis(300);
                    ext_led_kuning.set_low();
                    delay.delay_millis(300);
                }
                last_active_sensor = 2; // Tandai transisi ke Sensor 2 selesai
            }

            // ==========================================
            // TAHAP 2: BACA SENSOR 2 (Hanya jika S1 Mati)
            // ==========================================
            trigger2.set_high();
            delay.delay_micros(10);
            trigger2.set_low();

            timeout = Instant::now();
            let mut s2_failed = false;
            while echo2.is_low() {
                if timeout.elapsed() > Duration::from_millis(40) {
                    s2_failed = true;
                    break;
                }
            }
            if !s2_failed {
                let echo_start2 = Instant::now();
                while echo2.is_high() {
                    if echo_start2.elapsed() > Duration::from_millis(40) {
                        s2_failed = true;
                        break;
                    }
                }
                if !s2_failed {
                    dist2 = (echo_start2.elapsed().as_micros() as f32 * 0.0343) / 2.0;
                }
            }

            // Evaluasi Sensor 2 (Dibatasi maksimal hanya sampai 100.0 cm)
            if dist2 >= 2.0 && dist2 <= 100.0 {
                active_sensor = 2;
                final_distance = dist2;
                if last_active_sensor == 3 {
                    last_active_sensor = 2; // Pulihkan ke state 2 jika Sensor 2 pulih kembali
                }
            } else {
                // ==========================================
                // TRANSISI: SENSOR 2 MATI / MELEBIHI 100CM!
                // ==========================================
                if last_active_sensor == 2 {
                    ext_led_hijau.set_low();
                    ext_led_kuning.set_low();

                    // Berkedip 7 kali dalam 3 detik (214ms Nyala, 214ms Mati)
                    for _ in 0..7 {
                        ext_led_merah.set_high();
                        delay.delay_millis(214);
                        ext_led_merah.set_low();
                        delay.delay_millis(214);
                    }
                    last_active_sensor = 3; // Tandai transisi ke Sensor 3 selesai
                }

                // ==========================================
                // TAHAP 3: BACA SENSOR 3 (Hanya jika S1 & S2 Mati)
                // ==========================================
                trigger3.set_high();
                delay.delay_micros(10);
                trigger3.set_low();

                timeout = Instant::now();
                let mut s3_failed = false;
                while echo3.is_low() {
                    if timeout.elapsed() > Duration::from_millis(40) {
                        s3_failed = true;
                        break;
                    }
                }
                if !s3_failed {
                    let echo_start3 = Instant::now();
                    while echo3.is_high() {
                        if echo_start3.elapsed() > Duration::from_millis(40) {
                            s3_failed = true;
                            break;
                        }
                    }
                    if !s3_failed {
                        dist3 = (echo_start3.elapsed().as_micros() as f32 * 0.0343) / 2.0;
                    }
                }

                // Evaluasi Sensor 3
                if dist3 >= 2.0 && dist3 <= 400.0 {
                    active_sensor = 3;
                    final_distance = dist3;
                } else {
                    active_sensor = 0; // Semua Sensor Mati!
                }
            }
        }

        // ==========================================
        // TAHAP 4: KONTROL INDIKATOR SETELAH PEMBACAAN
        // ==========================================
        if active_sensor == 0 {
            // Semua sensor mati
            led_builtin.write(brightness(core::iter::once(RGB8::new(0, 0, 255)), 10)).unwrap(); // RGB Biru
            ext_led_hijau.set_low();
            ext_led_kuning.set_low();
            ext_led_merah.set_low();
            
            // Format angka koma murni saat error
            println!("0.00,0.00,0,0,0");
        } else {
            // Hitung Ketinggian Air menggunakan persamaan linier (12cm = 76%, 0cm = 100%)
            let mut level_percent = 100.0 - (2.0 * final_distance);
            if level_percent < 0.0 { 
                level_percent = 0.0; 
            }

            let builtin_color;
            let mut led_8 = 0;
            let mut led_10 = 0;
            let mut led_9 = 0;

            if final_distance < 6.0 {
                // KONDISI SIS (Shutdown): Di bawah 6 cm -> LED Merah Aktif (Pin 9)
                builtin_color = RGB8::new(255, 0, 0); // RGB Merah
                led_9 = 1;
            } else if final_distance >= 6.0 && final_distance < 12.0 {
                // KONDISI ALARM: Antara 12 - 6 cm -> LED Kuning Aktif (Pin 10)
                builtin_color = RGB8::new(255, 255, 0); // RGB Kuning
                led_10 = 1;
            } else {
                // KONDISI BPCS (Normal): Di atas 12 cm -> LED Hijau Aktif (Pin 8 jika <= 20 cm)
                builtin_color = RGB8::new(0, 255, 0); // RGB Hijau

                if final_distance <= 20.0 {
                    led_8 = 1;
                }
            }

            // Terapkan Logika Biner ke LED Eksternal Fisik
            if led_8 == 1 { ext_led_hijau.set_high(); } else { ext_led_hijau.set_low(); }
            if led_9 == 1 { ext_led_merah.set_high(); } else { ext_led_merah.set_low(); }
            if led_10 == 1 { ext_led_kuning.set_high(); } else { ext_led_kuning.set_low(); }

            // Terapkan ke RGB Internal
            led_builtin.write(brightness(core::iter::once(builtin_color), 10)).unwrap();

            // Cetak format angka murni dipisahkan koma sesuai permintaan Anda [input_file_8.png, input_file_9.png]
            println!(
                "{:.2},{:.2},{},{},{}",
                final_distance, level_percent, led_8, led_10, led_9
            );
        }

        // Delay 1 detik sebelum siklus pembacaan berikutnya
        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(1000) {}
    }
}

// Fungsi pembantu untuk konversi status ke tulisan teks (tetap dipertahankan untuk referensi)
fn s_state_to_str(state: u8) -> &'static str {
    match state {
        1 => "BPCS (Normal)",
        2 => "ALARM (Warning)",
        3 => "SIS (Shutdown)",
        _ => "Offline",
    }
}