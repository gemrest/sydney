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

//! <https://github.com/fdehau/tui-rs/blob/master/examples/list.rs>

use tui::widgets::ListState;

pub struct StatefulList<T> {
  pub state:    ListState,
  pub items:    Vec<T>,
  pub selected: usize,
}

impl<T> StatefulList<T> {
  pub fn with_items(items: Vec<T>) -> Self {
    Self {
      state: ListState::default(),
      items,
      selected: 0,
    }
  }

  pub fn next(&mut self) {
    let i = match self.state.selected() {
      Some(i) =>
        if i >= self.items.len() - 1 {
          0
        } else {
          i + 1
        },
      None => 0,
    };

    self.selected = i;

    self.state.select(Some(i));
  }

  pub fn previous(&mut self) {
    let i = match self.state.selected() {
      Some(i) =>
        if i == 0 {
          self.items.len() - 1
        } else {
          i - 1
        },
      None => 0,
    };

    self.selected = i;

    self.state.select(Some(i));
  }

  pub fn last(&mut self) {
    self.state.select(Some(self.items.len() - 1));

    self.selected = self.items.len() - 1;
  }

  pub fn first(&mut self) {
    self.state.select(Some(0));

    self.selected = 0;
  }

  pub fn unselect(&mut self) { self.state.select(None); }
}
