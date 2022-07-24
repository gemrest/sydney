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

use germ::ast::Node;
use tui::{
  layout::{Constraint, Direction, Layout, Rect},
  style::{Color, Modifier, Style},
  text::{Span, Spans},
  widgets,
  widgets::{ListItem, Paragraph},
};

#[allow(clippy::too_many_lines)]
pub fn ui<B: tui::backend::Backend>(
  f: &mut tui::Frame<'_, B>,
  app: &mut crate::App,
) {
  let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints(
      [
        Constraint::Percentage(95),
        Constraint::Percentage(4),
        Constraint::Percentage(1),
      ]
      .as_ref(),
    )
    .split(f.size());

  let items: Vec<ListItem<'_>> = app
    .items
    .items
    .iter()
    .map(|(text_lines, _link, pre)| {
      let mut spans = vec![];

      for line in text_lines {
        let mut line = line.clone();

        if *pre {
          if let Node::Text(text) = line {
            line = Node::PreformattedText {
              alt_text: None,
              text:     text.to_string(),
            }
          }
        }

        match line {
          germ::ast::Node::Text(text) =>
            if text != "sydney_abc_123" {
              spans.push(tui::text::Spans::from(format!("    {}", text)));
            },
          germ::ast::Node::Blockquote(text) => {
            spans.push(Spans::from(vec![
              Span::styled("  > ", Style::default().fg(Color::LightBlue)),
              Span::styled(
                text,
                Style::default().add_modifier(Modifier::ITALIC),
              ),
            ]));
          }
          germ::ast::Node::Link {
            to,
            text,
          } => {
            let mut span_list =
              vec![Span::styled(" => ", Style::default().fg(Color::LightBlue))];

            span_list.push(Span::styled(
              text.unwrap_or_else(|| to.clone()),
              Style::default().add_modifier(Modifier::UNDERLINED),
            ));
            span_list.push(Span::from(" "));
            span_list
              .push(Span::styled(to, Style::default().fg(Color::LightBlue)));

            spans.push(Spans::from(span_list));
          }
          germ::ast::Node::Heading {
            text,
            level,
          } => {
            spans.push(Spans::from(vec![
              Span::styled(
                match level {
                  1 => "  # ",
                  2 => " ## ",
                  3 => "### ",
                  _ => unreachable!(),
                },
                Style::default().fg(Color::LightBlue),
              ),
              Span::styled(text, {
                let mut style = Style::default().add_modifier(Modifier::BOLD);

                match level {
                  1 => {
                    style = style.add_modifier(Modifier::UNDERLINED);
                  }
                  3 => {
                    style = style.add_modifier(Modifier::ITALIC);
                  }
                  _ => {}
                }

                style
              }),
            ]));
          }
          germ::ast::Node::List(list_items) => {
            let mut span_list = vec![];

            for list_item in list_items {
              span_list.push(Span::styled(
                "  * ",
                Style::default().fg(Color::LightBlue),
              ));
              span_list.push(Span::from(format!("{}\n", list_item)));
            }

            spans.push(Spans::from(span_list));
          }
          germ::ast::Node::PreformattedText {
            text,
            alt_text,
          } => {
            let mut span_list = vec![];

            span_list.push(Span::styled(
              "``` ",
              Style::default().fg(Color::LightBlue),
            ));
            span_list
              .push(Span::from(alt_text.unwrap_or_else(|| "".to_string())));

            if text != "sydney_abc_123" {
              span_list.push(Span::from(text));
            }

            spans.push(Spans::from(span_list));
          }
          germ::ast::Node::Whitespace => {
            spans.push(Spans::from("".to_string()));
          }
        };
      }

      ListItem::new(spans)
    })
    .collect();

  let items = widgets::List::new(items)
    .highlight_style(
      Style::default()
        .bg(Color::White)
        .fg(Color::Black)
        .remove_modifier(Modifier::BOLD),
    )
    .style(Style::default().bg(Color::Black).fg(Color::White));

  f.render_stateful_widget(items, chunks[0], &mut app.items.state);
  f.render_widget(
    Paragraph::new(app.url.to_string())
      .style(Style::default().bg(Color::White).fg(Color::Black)),
    chunks[1],
  );

  if let Some(error) = app.error.as_ref() {
    f.render_widget(
      Paragraph::new(&**error).style(Style::default().bg(Color::Red)),
      chunks[2],
    );
  } else if !app.input.is_empty()
    || app.input_mode == crate::input::Mode::Editing
  {
    f.render_widget(Paragraph::new(format!(":{}", app.input)), chunks[2]);
  }

  if app.accept_response_input {
    let block = widgets::Block::default()
      .title(app.url.to_string())
      .borders(widgets::Borders::ALL);
    let area = centered_rect(60, 20, f.size());

    f.render_widget(widgets::Clear, area);
    f.render_widget(block.clone(), area);
    f.render_widget(
      Paragraph::new(format!(
        "{} {}",
        app.response_input_text.trim(),
        app.response_input
      ))
      .wrap(widgets::Wrap {
        trim: false
      }),
      block.inner(area),
    );
  }

  if let Some(error) = &app.error {
    let block = widgets::Block::default()
      .title("Sydney")
      .borders(widgets::Borders::ALL)
      .style(Style::default().bg(Color::Cyan));
    let area = centered_rect(60, 20, f.size());

    f.render_widget(widgets::Clear, area);
    f.render_widget(block.clone(), area);
    f.render_widget(
      Paragraph::new(error.to_string()).wrap(widgets::Wrap {
        trim: false
      }),
      block.inner(area),
    );
  }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
  let popup_layout = Layout::default()
    .direction(Direction::Vertical)
    .constraints(
      [
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
      ]
      .as_ref(),
    )
    .split(r);

  Layout::default()
    .direction(Direction::Horizontal)
    .constraints(
      [
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
      ]
      .as_ref(),
    )
    .split(popup_layout[1])[1]
}
