use crate::errors::*;
use std::cmp;
use std::fmt::Display;
use crate::models::application::modes::{SearchSelectMode};
use pad::PadStr;
use crate::presenters::current_buffer_status_line_data;
use scribe::Workspace;
use scribe::buffer::Position;
use crate::view::{Colors, StatusLineData, Style, View};
use unicode_segmentation::UnicodeSegmentation;

pub fn display<T: Display>(workspace: &mut Workspace, mode: &mut SearchSelectMode<T>, view: &mut View) -> Result<()> {
    let mut presenter = view.build_presenter()?;
    let mode_config = mode.config().clone();

    // Wipe the slate clean.
    presenter.clear();

    let buffer_status = current_buffer_status_line_data(workspace);

    if let Some(buf) = workspace.current_buffer() {
        presenter.draw_buffer(buf, None, None)?;

        // Draw the status line.
        presenter.draw_status_line(&[
            StatusLineData {
                content: format!(" {} ", mode),
                style: Style::Default,
                colors: Colors::Inverted,
            },
            buffer_status
        ]);
    }

    if let Some(message) = mode.message() {
        presenter.print(&Position{ line: 0, offset: 0 },
                   Style::Default,
                   Colors::Default,
                   &message.pad_to_width(presenter.width()))?;
    } else {
        // Draw the list of search results.
        for (line, result) in mode.results().enumerate() {
            let (content, colors, style) = if line == mode.selected_index() {
                (format!("> {}", result), Colors::Focused, Style::Bold)
            } else {
                (format!("  {}", result), Colors::Default, Style::Default)
            };
            let padded_content = content.pad_to_width(presenter.width());
            presenter.print(&Position{ line, offset: 0 },
                       style,
                       colors,
                       &padded_content)?;
        }
    }

    // Clear any remaining lines in the result display area.
    for line in cmp::max(mode.results().len(), 1)..mode_config.max_results {
        presenter.print(&Position{ line, offset: 0 },
                   Style::Default,
                   Colors::Default,
                   &String::new().pad_to_width(presenter.width()))?;
    }

    // Draw the divider.
    let line = mode_config.max_results;
    let colors = if mode.insert_mode() {
        Colors::Insert
    } else {
        Colors::Inverted
    };
    let padded_content = mode.query().pad_to_width(presenter.width());
    presenter.print(&Position{ line, offset: 0 },
               Style::Bold,
               colors,
               &padded_content)?;

    // Place the cursor on the search input line, right after its contents.
    presenter.set_cursor(Some(Position {
        line: mode_config.max_results,
        offset: mode.query().graphemes(true).count(),
    }));

    // Render the changes to the screen.
    presenter.present();

    Ok(())
}
