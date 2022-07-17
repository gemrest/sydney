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

use std::time::{Duration, Instant};

use crossterm::event;
use germ::request::Status;
use url::Url;

use crate::{input::Mode as InputMode, stateful_list::StatefulList};

pub struct App {
  pub items:                  StatefulList<(Vec<String>, Option<String>)>,
  pub input:                  String,
  pub input_mode:             InputMode,
  pub command_stroke_history: Vec<event::KeyCode>,
  pub command_history:        Vec<String>,
  pub command_history_cursor: usize,
  pub error:                  Option<String>,
  pub url:                    Url,
  pub capsule_history:        Vec<Url>,
  pub previous_capsule:       Option<Url>,
  pub response_input:         String,
  pub accept_response_input:  bool,
  pub response_input_text:    String,
  pub wrap_at:                usize,
}
impl App {
  pub fn new() -> Self {
    let url = Url::parse("gemini://gemini.circumlunar.space/").unwrap();

    let mut app = Self {
      response_input: String::new(),
      error: None,
      command_stroke_history: Vec::new(),
      input: String::new(),
      input_mode: InputMode::Normal,
      items: StatefulList::with_items(Vec::new()),
      command_history: vec![],
      command_history_cursor: 0,
      url,
      capsule_history: vec![],
      previous_capsule: None,
      accept_response_input: false,
      response_input_text: "".to_string(),
      wrap_at: 80,
    };

    app.make_request();

    app
  }

  pub fn set_url(&mut self, url: Url) {
    self.previous_capsule = Some(self.url.clone());
    self.url = url;
  }

  pub fn make_request(&mut self) {
    self.items = StatefulList::with_items({
      let mut items = vec![];

      match germ::request::request(&self.url) {
        Ok(mut response) => {
          if response.status() == &Status::TemporaryRedirect
            || response.status() == &Status::PermanentRedirect
          {
            self.url = Url::parse(&if response.meta().starts_with('/') {
              format!(
                "gemini://{}{}",
                self.url.host_str().unwrap(),
                response.meta()
              )
            } else if response.meta().starts_with("gemini://") {
              response.meta().to_string()
            } else if !response.meta().starts_with('/')
              && !response.meta().starts_with("gemini://")
            {
              format!(
                "{}/{}",
                self.url.to_string().trim_end_matches('/'),
                response.meta()
              )
            } else {
              self.url.to_string()
            })
            .unwrap();

            response = germ::request::request(&self.url).unwrap();
          }

          if response.status() == &Status::Input
            || response.status() == &Status::SensitiveInput
          {
            self.accept_response_input = true;
            self.response_input_text = response.meta().to_string();
            items = self.items.items.clone();
          }

          items.push((vec![format!("{}", response.meta().to_string())], None));
          items.push((vec!["".to_string()], None));

          if let Some(content) = response.content().clone() {
            for line in content.lines().clone() {
              let line = line.replace('\t', " ");
              let mut parts = line.split_whitespace();
              let lines = if line.is_empty() {
                vec![line.to_string()]
              } else {
                line
                  .as_bytes()
                  .chunks(self.wrap_at)
                  .map(|buf| {
                    #[allow(unsafe_code)]
                    unsafe { std::str::from_utf8_unchecked(buf) }.to_string()
                  })
                  .collect::<Vec<_>>()
              };

              if let (Some("=>"), Some(to)) = (parts.next(), parts.next()) {
                items.push((lines, Some(to.to_string())));
              } else {
                items.push((lines, None));
              }
            }
          } else if response.status() != &Status::Input
            && response.status() != &Status::SensitiveInput
          {
            items.push((vec![response.meta().to_string()], None));

            self.error = Some(response.meta().to_string());
          }

          if let Some(last_url) = self.capsule_history.last() {
            if last_url.to_string() != self.url.to_string() {
              self.capsule_history.push(self.url.clone());
            }
          } else {
            self.capsule_history.push(self.url.clone());
          }
        }
        Err(error) => {
          self.error = Some(error.to_string());

          return;
        }
      }

      items
    });
  }

  pub fn run<B: tui::backend::Backend>(
    terminal: &mut tui::Terminal<B>,
    mut app: Self,
    tick_rate: Duration,
  ) -> std::io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
      terminal.draw(|f| crate::ui::ui(f, &mut app))?;

      let timeout = tick_rate
        .checked_sub(last_tick.elapsed())
        .unwrap_or_else(|| Duration::from_secs(0));

      if event::poll(timeout)? {
        if let event::Event::Key(key) = event::read()? {
          if crate::input::handle_key_strokes(&mut app, key) {
            return Ok(());
          }
        }
      }

      if last_tick.elapsed() >= tick_rate {
        last_tick = Instant::now();
      }
    }
  }

  pub fn go_back(&mut self) {
    if let Some(url) = self.capsule_history.pop() {
      if url == self.url {
        if let Some(url) = self.capsule_history.pop() {
          self.set_url(url);
        }
      } else {
        self.set_url(url);
      }

      self.make_request();
    }
  }
}
