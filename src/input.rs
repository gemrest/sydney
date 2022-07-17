// This file is part of Germ <https://github.com/gemrest/sydney>.
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

use crossterm::event::KeyCode;
use url::Url;

use crate::command::Command;

#[derive(PartialEq)]
pub enum Mode {
  Normal,
  Editing,
}

fn handle_input_response(
  mut app: &mut crate::App,
  key: crossterm::event::KeyEvent,
) -> bool {
  match key.code {
    KeyCode::Enter => {
      let new_url = match app.url.to_string().split('?').next() {
        Some(base_url) => {
          format!("{}?{}", base_url, app.response_input)
        }
        None => "".to_string(),
      };

      if new_url.is_empty() {
        app.error = Some("Invalid base URL".to_string());

        return false;
      }

      match Url::parse(&new_url) {
        Ok(url) => {
          app.set_url(url);
          app.make_request();
          app.response_input.clear();
          app.response_input_text.clear();

          app.accept_response_input = false;
        }
        Err(error) => {
          app.error = Some(error.to_string());
        }
      }
    }
    KeyCode::Esc => {
      app.accept_response_input = false;

      app.response_input.clear();
      app.response_input_text.clear();
      app.go_back();
    }
    KeyCode::Char(c) => {
      app.response_input.push(c);
    }
    KeyCode::Backspace => {
      app.response_input.pop();
    }
    _ => {}
  }

  false
}

fn handle_normal_input(
  mut app: &mut crate::App,
  key: crossterm::event::KeyEvent,
) -> bool {
  match key.code {
    KeyCode::Char(':') => {
      app.input.clear();

      app.input_mode = Mode::Editing;
      app.error = None;
    }
    KeyCode::Esc => app.items.unselect(),
    KeyCode::Down | KeyCode::Char('j') => {
      app.items.next();

      app.error = None;
    }
    KeyCode::Up | KeyCode::Char('k') => {
      app.items.previous();

      app.error = None;
    }
    KeyCode::Char('h') | KeyCode::Left => {
      app.go_back();
    }
    KeyCode::Char('l') | KeyCode::Right => {
      if let Some(url) = app.previous_capsule.clone() {
        app.set_url(url);

        app.previous_capsule = None;

        app.make_request();
      }
    }
    KeyCode::Char('G') => app.items.last(),
    KeyCode::Char('g') =>
      if app.command_stroke_history.contains(&key.code) {
        app.items.first();
        app.command_stroke_history.clear();
      } else if app.command_stroke_history.is_empty() {
        app.command_stroke_history.push(key.code);
      },
    KeyCode::Backspace => app.error = None,
    KeyCode::Enter => {
      app.error = None;

      if let Some(link) = &app.items.items[app.items.selected].1 {
        if !link.starts_with("gemini://") && link.contains("://") {
        } else {
          let the_url = &if link.starts_with('/') {
            format!("gemini://{}{}", app.url.host_str().unwrap(), link)
          } else if link.starts_with("gemini://") {
            link.to_string()
          } else if !link.starts_with('/') && !link.starts_with("gemini://") {
            format!("{}/{}", app.url.to_string().trim_end_matches('/'), link)
          } else {
            app.url.to_string()
          };

          app.set_url(Url::parse(the_url).unwrap());
          app.make_request();
        }
      }
    }
    _ => {}
  }

  false
}

fn handle_editing_input(
  mut app: &mut crate::App,
  key: crossterm::event::KeyEvent,
) -> bool {
  match key.code {
    KeyCode::Enter => {
      app.command_history.reverse();
      app.command_history.push(app.input.to_string());
      app.command_history.reverse();

      match Command::from(app.input.to_string()) {
        Command::Quit => return true,
        Command::Open(to) =>
          if let Some(to) = to {
            app.set_url(
              Url::parse(&if to.starts_with("gemini://") {
                to
              } else {
                format!("gemini://{}", to)
              })
              .unwrap(),
            );

            app.make_request();
          } else {
            app.error = Some("No URL provided for open command".to_string());
          },
        Command::Unknown => {
          app.error = Some(format!("\"{}\" is not a valid command", app.input));
        }
        Command::Wrap(at, error) =>
          if let Some(error) = error {
            app.error = Some(error);
          } else {
            app.error = None;
            app.wrap_at = at;

            app.make_request();
          },
      }

      app.input_mode = Mode::Normal;
      app.command_history_cursor = 0;
    }
    KeyCode::Char(c) => {
      app.input.push(c);
    }
    KeyCode::Up => {
      if let Some(command) = app.command_history.get(app.command_history_cursor)
      {
        app.input = command.to_string();

        if app.command_history_cursor + 1 < app.command_history.len() {
          app.command_history_cursor += 1;
        }
      }
    }
    KeyCode::Down => {
      let mut dead_set = false;

      if app.command_history_cursor > 0 {
        app.command_history_cursor -= 1;
      } else {
        dead_set = true;
      }

      if let Some(command) = app.command_history.get(app.command_history_cursor)
      {
        app.input = command.to_string();
      }

      if dead_set {
        app.input.clear();
      }
    }
    KeyCode::Backspace => {
      app.input.pop();
    }
    KeyCode::Esc => {
      app.input_mode = Mode::Normal;

      app.input.clear();
    }
    _ => {}
  }

  false
}

pub fn handle_key_strokes(
  app: &mut crate::App,
  key: crossterm::event::KeyEvent,
) -> bool {
  match app.input_mode {
    Mode::Normal =>
      if app.accept_response_input {
        handle_input_response(app, key)
      } else {
        handle_normal_input(app, key)
      },
    Mode::Editing => handle_editing_input(app, key),
  }
}
