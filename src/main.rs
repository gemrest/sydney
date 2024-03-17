// This file is part of Sydney <https://github.com/gemrest/sydney>.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.
//
// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.
//
// Copyright (C) 2022-2022 Fuwn <contact@fuwn.me>
// SPDX-License-Identifier: GPL-3.0-only

#![deny(
  warnings,
  nonstandard_style,
  unused,
  future_incompatible,
  rust_2018_idioms,
  unsafe_code,
  clippy::all,
  clippy::nursery,
  clippy::pedantic
)]
#![recursion_limit = "128"]

mod app;
mod command;
mod input;
mod stateful_list;
mod ui;
mod url;

use ::url::Url;
use app::App;
use crossterm::{event, execute, terminal};

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let mut args = std::env::args();
  let mut app = App::new();

  if let Some(arg) = args.nth(1) {
    match arg.as_str() {
      "--version" | "-v" => {
        println!("{}", env!("CARGO_PKG_VERSION"));

        return Ok(());
      }
      "--help" | "-h" => {
        println!(
          r#"usage: {} [option, capsule_uri]
Options:
    --version, -v    show version text
    --help, -h       show help text

Sample invocations:
    {0} gemini://gem.rest/
    {0} --help

Report bugs to https://github.com/gemrest/sydney/issues"#,
          args
            .next()
            .unwrap_or_else(|| env!("CARGO_PKG_NAME").to_string())
        );

        return Ok(());
      }
      _ => {
        app.url = Url::parse(&url::prefix_gemini(&arg))?;

        app.make_request();
      }
    }
  }

  terminal::enable_raw_mode()?;

  let mut stdout = std::io::stdout();

  match germ::request::request(
    &Url::parse("gemini://fuwn.me/api/sydney/version").unwrap(),
  ) {
    Ok(response) =>
      if let Some(content) = response.content() {
        let content = content.trim();

        if content > &String::from(env!("CARGO_PKG_VERSION")) {
          app.error = Some(format!(
            "Your Sydney version ({}) is outdated. It is recommended that you \
             update to the newest version ({}).",
            env!("CARGO_PKG_VERSION"),
            content,
          ));
        }
      } else {
        app.error = Some(
          "Could not check if Sydney has a newer version because the response \
           had no content. Please try again later."
            .to_string(),
        );
      },
    Err(error) =>
      app.error = Some(format!(
        "Could not check if Sydney has a newer version: {}",
        error
      )),
  }

  execute!(
    stdout,
    terminal::EnterAlternateScreen,
    event::EnableMouseCapture
  )?;

  let mut terminal =
    ratatui::Terminal::new(ratatui::backend::CrosstermBackend::new(stdout))?;
  let result =
    App::run(&mut terminal, app, std::time::Duration::from_millis(250));

  terminal::disable_raw_mode()?;
  execute!(
    terminal.backend_mut(),
    terminal::LeaveAlternateScreen,
    event::DisableMouseCapture
  )?;
  terminal.show_cursor()?;

  if let Err(err) = result {
    println!("{:?}", err);
  }

  Ok(())
}
