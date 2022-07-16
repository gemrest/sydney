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

pub enum Command {
  Quit,
  Open(Option<String>),
  Unknown,
  Wrap(usize, Option<String>),
}
impl From<String> for Command {
  fn from(s: String) -> Self {
    let mut tokens = s.split(' ');

    match tokens.next() {
      Some("open" | "o") => Self::Open(tokens.next().map(ToString::to_string)),
      Some("quit" | "q") => Self::Quit,
      Some("wrap") =>
        tokens.next().map_or_else(
          || {
            Self::Wrap(
              80,
              Some("Missing width argument to wrap command".to_string()),
            )
          },
          |at| {
            match at.parse() {
              Ok(at_parsed) => Self::Wrap(at_parsed, None),
              Err(error) => Self::Wrap(80, Some(error.to_string())),
            }
          },
        ),
      _ => Self::Unknown,
    }
  }
}
