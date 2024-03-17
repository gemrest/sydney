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

use std::time::{Duration, Instant};

use crossterm::event;
use germ::{ast::Node, request::Status};
use url::Url;

use crate::{input::Mode as InputMode, stateful_list::StatefulList};

pub struct App {
  pub items:                  StatefulList<(Vec<Node>, Option<String>, bool)>,
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
  pub wrap_at:                u16,
}
impl App {
  pub fn new() -> Self {
    let url = Url::parse("gemini://gem.rest/projects/sydney.gmi").unwrap();

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
      wrap_at: crossterm::terminal::size().unwrap_or((80, 24)).0,
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
      let mut items: Vec<(Vec<Node>, Option<String>, bool)> = vec![];

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

          // items.push((
          //   vec![Node::Text(response.meta().to_string())],
          //   None,
          //   false,
          // ));
          // items.push((vec![Node::Text("".to_string())], None, false));

          let mut pre = false;

          if let Some(content) = response.content().clone() {
            let real_lines = content.lines();

            for line in real_lines {
              let line = line.replace('\t', " ");
              let pre_like = if line.starts_with("```") {
                pre = !pre;

                true
              } else {
                false
              };

              let ast = germ::ast::Ast::from_string(&line);
              let ast_node = ast.inner().first().map_or_else(
                || {
                  if pre_like || pre {
                    if line == "```" {
                      Node::Text("sydney_abc_123".to_string())
                    } else {
                      Node::Text(line.get(3..).unwrap_or("").to_string())
                    }
                  } else {
                    Node::Whitespace
                  }
                },
                Clone::clone,
              );

              let mut parts = line.split_whitespace();

              if let (Some("=>"), Some(to)) = (parts.next(), parts.next()) {
                items.push((vec![ast_node], Some(to.to_string()), false));
              } else {
                items.push((vec![ast_node], None, pre));
              }
            }
          } else if response.status() != &Status::Input
            && response.status() != &Status::SensitiveInput
          {
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

  pub fn run<B: ratatui::backend::Backend>(
    terminal: &mut ratatui::Terminal<B>,
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
