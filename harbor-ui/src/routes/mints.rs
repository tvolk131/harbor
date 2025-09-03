use std::str::FromStr;

use harbor_client::MintConnectionInfo;
use iced::Element;
use iced::widget::{column, row};

use crate::components::{
    InputArgs, SvgIcon, basic_layout, h_button, h_federation_archived, h_federation_item,
    h_federation_item_preview, h_header, h_input, operation_status_for_id, subtitle, the_spinner,
};
use crate::{AddFederationStatus, HarborWallet, Message, PeekStatus};

use super::{MintSubroute, Route};

// Expects to always have at least one federation, otherwise we should be on the add mint screen
// TODO: now that we have archived mints, we should show them even if there are no active mints
fn mints_list(harbor: &HarborWallet) -> Element<Message> {
    let header = h_header("Mints", "Manage your mints here.");

    let active = harbor
        .mint_list
        .iter()
        .filter(|a| a.active)
        .fold(column![], |column, item| {
            column.push(h_federation_item(item))
        })
        .spacing(48);

    let inactive = harbor
        .mint_list
        .iter()
        .filter(|a| !a.active)
        .fold(column![], |column, item| {
            column.push(h_federation_archived(item, harbor))
        })
        .spacing(48);

    let add_another_mint_button = h_button("Add Another Mint", SvgIcon::Plus, false)
        .on_press(Message::Navigate(Route::Mints(MintSubroute::Add)));

    let discover_mints_button = h_button("Discover Mints", SvgIcon::Eye, false)
        .on_press(Message::Navigate(Route::Mints(MintSubroute::Discover)));

    // if we have inactive mints, display them
    let column = if harbor.mint_list.iter().filter(|a| !a.active).count() > 0 {
        let archived_header = h_header("Archived Mints", "Mints you've joined and left.");
        column![
            header,
            active,
            add_another_mint_button,
            discover_mints_button,
            archived_header,
            inactive
        ]
        .spacing(48)
    } else {
        column![
            header,
            active,
            add_another_mint_button,
            discover_mints_button
        ]
        .spacing(48)
    };

    basic_layout(column)
}

fn mints_add(harbor: &HarborWallet) -> Element<Message> {
    let header = h_header("Add Mint", "Add a new mint to your wallet.");

    let mint_connection_info = MintConnectionInfo::from_str(&harbor.mint_invite_code_str).ok();

    let column = match &harbor.peek_federation_item {
        None => {
            let mint_input = h_input(InputArgs {
                label: "Invite Code or Mint URL",
                value: &harbor.mint_invite_code_str,
                on_input: Message::MintInviteCodeInputChanged,
                disabled: harbor.peek_status == PeekStatus::Peeking,
                ..InputArgs::default()
            });

            let peek_mint_button = h_button(
                "Preview",
                SvgIcon::Eye,
                harbor.peek_status == PeekStatus::Peeking,
            )
            .on_press_maybe(mint_connection_info.map(Message::PeekMint));

            let mut peek_column = column![mint_input, peek_mint_button].spacing(16);

            // Add status display for preview operation
            if let Some(current_peek_id) = harbor.current_peek_id {
                if let Some(status) = operation_status_for_id(harbor, Some(current_peek_id)) {
                    peek_column = peek_column.push(status);
                }
            }

            column![header, peek_column].spacing(48)
        }

        Some(peek_federation_item) => {
            let federation_preview = h_federation_item_preview(peek_federation_item);

            let is_joining = harbor.add_federation_status == AddFederationStatus::Adding;

            let add_mint_button = h_button("Join Mint", SvgIcon::Plus, is_joining)
                .on_press_maybe(mint_connection_info.map(Message::AddMint));

            let start_over_button = h_button("Start Over", SvgIcon::Restart, false)
                .on_press(Message::CancelAddFederation);

            let button_row = row![start_over_button, add_mint_button].spacing(16);
            let mut preview_column = column![federation_preview, button_row].spacing(16);

            // Add status display for add operation
            if let Some(current_add_id) = harbor.current_add_id {
                if let Some(status) = operation_status_for_id(harbor, Some(current_add_id)) {
                    preview_column = preview_column.push(status);
                }
            }

            column![header, preview_column].spacing(48)
        }
    };

    basic_layout(column)
}

fn mints_discover(harbor: &HarborWallet) -> Element<Message> {
    let list = if !harbor.discovered_mints.is_empty() {
        harbor
            .discovered_mints
            .iter()
            .fold(column![], |column, mint| {
                let preview = h_federation_item_preview(&mint.item);
                let add_mint_button = h_button(
                    "Join Mint",
                    SvgIcon::Plus,
                    harbor.add_federation_status == AddFederationStatus::Adding,
                )
                .on_press(Message::AddMint(mint.join_str.clone()));
                column.push(column![preview, add_mint_button].spacing(16))
            })
            .spacing(48)
    } else if !harbor.discover_pending.is_empty() {
        column![the_spinner()].spacing(16)
    } else {
        column![
            iced::widget::text("No mints found")
                .size(18)
                .style(subtitle)
        ]
        .spacing(16)
    };

    basic_layout(
        column![
            h_header("Discover Mints", "Find mints announced on Nostr."),
            list
        ]
        .spacing(48),
    )
}

pub fn mints(harbor: &HarborWallet) -> Element<Message> {
    if harbor.mint_list.iter().filter(|f| f.active).count() == 0 {
        match harbor.active_route {
            Route::Mints(MintSubroute::Discover) => mints_discover(harbor),
            _ => mints_add(harbor),
        }
    } else {
        match harbor.active_route {
            Route::Mints(MintSubroute::Add) => mints_add(harbor),
            Route::Mints(MintSubroute::Discover) => mints_discover(harbor),
            _ => mints_list(harbor),
        }
    }
}
