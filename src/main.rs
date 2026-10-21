/*
 * csleep - Chunked sleep and displays a progress bar
 * Copyright (C) 2026 Nimesh Poudel
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
 * See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */
use std::env;
use std::io::{self, Write};
use std::process;
use std::thread;
use std::time::{Duration, Instant};
use terminal_size::{Width, terminal_size};
const USAGE: &str = "Usage: csleep Number[Suffix]
or: csleep OPTION
Pause for NUMBER seconds, where NUMBER is an integer or floating-point.
SUFFIX may be 's','m','h', or 'd', for seconds, minutes, hours, days.
With multiple arguments, pause for the sum of their values.
    --help
        display this help and exit
    --version
        output version information and exit
    --author
        output author name and exit
    --license
        output the full license text and exit";

const AUTHOR: &str = "Nimesh Poudel";
const LICENSE: &str = include_str!("../LICENSE.txt");

//const BAR_WIDTH: usize = 40;
// Eighth-block characters give 8 sub-steps per cell for a smoother bar.
const PARTIAL: [char; 8] = [' ', '▏', '▎', '▍', '▌', '▋', '▊', '▉'];

/// Parses one argument such as "90", "1.5m" or "2h3m3s" into a Duration.
/// A bare number (no suffix) is only allowed when it is the whole argument.
fn parse_duration(arg: &str) -> Result<Duration, String> {
    let err = || format!("invalid time interval '{arg}'");
    if arg.is_empty() {
        return Err(err());
    }

    let mut total = Duration::ZERO;
    let mut rest = arg;

    while !rest.is_empty() {
        let num_end = rest
            .find(|c: char| !(c.is_ascii_digit() || c == '.'))
            .unwrap_or(rest.len());
        let (num, tail) = rest.split_at(num_end);

        let mut chars = tail.chars();
        let mult = match chars.next() {
            Some('s') => 1.0,
            Some('m') => 60.0,
            Some('h') => 3600.0,
            Some('d') => 86400.0,
            // No suffix: only valid if this is the entire argument.
            None if rest.len() == arg.len() => 1.0,
            _ => return Err(err()),
        };
        rest = chars.as_str();

        let value: f64 = num.parse().map_err(|_| err())?;
        let part = Duration::try_from_secs_f64(value * mult).map_err(|_| err())?;
        total = total.checked_add(part).ok_or_else(err)?;
    }
    Ok(total)
}

fn fmt_hms(secs: u64) -> String {
    format!(
        "{:02}:{:02}:{:02}",
        secs / 3600,
        (secs / 60) % 60,
        secs % 60
    )
}

/// Width of the progress bar, leaving room for " XXX%  HH:MM:SS left".
fn bar_width() -> usize {
    const RESERVED: usize = 22; // " 100%  00:00:00 left"
    const MIN_WIDTH: usize = 10;
    const FALLBACK: usize = 40;

    match terminal_size() {
        Some((Width(w), _)) => (w as usize).saturating_sub(RESERVED).max(MIN_WIDTH),
        None => FALLBACK, // not a TTY, redirected, etc.
    }
}
fn draw_bar(elapsed: Duration, total: Duration) {
    let frac = (elapsed.as_secs_f64() / total.as_secs_f64()).clamp(0.0, 1.0);
    let cells = frac * bar_width() as f64;
    let full = cells.floor() as usize;
    let part = ((cells - full as f64) * 8.0) as usize;

    let mut bar = "█".repeat(full);
    if part > 0 {
        bar.push(PARTIAL[part]);
    }
    let used = full + usize::from(part > 0);
    bar.push_str(&"░".repeat(bar_width() - used));

    let left = total.saturating_sub(elapsed).as_secs_f64().ceil() as u64;
    eprint!(
        "\r{bar} {:>3}%  {} left",
        (frac * 100.0).round() as u32,
        fmt_hms(left)
    );
    io::stderr().flush().ok();
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        eprintln!("csleep: missing operand\nTry 'csleep --help' for more information.");
        process::exit(1);
    }

    // Options are handled first so they work regardless of position.
    for arg in &args {
        match arg.as_str() {
            "--help" => {
                println!("{USAGE}");
                return;
            }
            "--version" => {
                println!("csleep {}", env!("CARGO_PKG_VERSION"));
                return;
            }
            "--author" => {
                println!("{AUTHOR}");
                return;
            }
            "--license" => {
                print!("{LICENSE}");
                return;
            }
            _ => {}
        }
    }

    let mut total = Duration::ZERO;
    for arg in &args {
        let d = parse_duration(arg).unwrap_or_else(|e| {
            eprintln!("csleep: {e}\nTry 'csleep --help' for more information.");
            process::exit(1);
        });
        total = total.checked_add(d).unwrap_or_else(|| {
            eprintln!("csleep: total duration is too large");
            process::exit(1);
        });
    }

    if total.is_zero() {
        return;
    }

    let chunk = (total / 100).max(Duration::from_secs(1));
    let start = Instant::now();
    let mut target = Duration::ZERO;

    draw_bar(target, total);
    while target < total {
        target = target.checked_add(chunk).map_or(total, |t| t.min(total));
        let elapsed = start.elapsed();
        if target > elapsed {
            thread::sleep(target - elapsed);
        }
        draw_bar(target, total);
    }
    eprintln!();
}
