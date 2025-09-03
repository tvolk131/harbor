use std::{
    collections::{BTreeMap, BTreeSet},
    str::FromStr,
    time::Duration,
};

use harbor_client::{
    bitcoin::Network,
    fedimint_core::{config::FederationId, invite_code::InviteCode},
};
use nostr_sdk::{Alphabet, Client, Event, Filter, Keys, Kind, SingleLetterTag, TagKind};

const HARDCODED_RELAYS: [&str; 3] = [
    "wss://relay.damus.io",
    "wss://relay.primal.net",
    "wss://relay.snort.social",
];

const NIP87_MINT_RECOMMENDATION_KIND: u16 = 38_000;
const NIP87_MINT_ANNOUNCEMENT_CASHU_KIND: u16 = 38_172;
const NIP87_MINT_ANNOUNCEMENT_FEDIMINT_KIND: u16 = 38_173;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CashuAnnouncement {
    // TODO: Figure out what strongly-typed type this should be.
    pub mint_pubkey: String,
    // TODO: Figure out what strongly-typed type this should be.
    pub url: String,
    pub nuts: BTreeSet<u16>,
}

impl CashuAnnouncement {
    /// Merge & deduplicate a list of announcements such that the resulting set
    /// contains only one announcement per mint pubkey, the most commonly seen
    /// url, and all available nuts seen in any announcement for a given mint.
    fn aggregate(announcements: Vec<Self>) -> BTreeMap<String, Self> {
        let mut announcements_by_mint_pubkey: BTreeMap<String, Vec<Self>> = BTreeMap::new();
        for announcement in announcements {
            announcements_by_mint_pubkey
                .entry(announcement.mint_pubkey.clone())
                .or_default()
                .push(announcement);
        }

        announcements_by_mint_pubkey
            .into_iter()
            .filter_map(|(mint_pubkey, announcements)| {
                let most_common_url = get_most_common_string(
                    announcements
                        .iter()
                        .map(|a| a.url.as_str())
                        .collect::<Vec<_>>()
                        .as_slice(),
                )?
                .to_string();

                let mut all_seen_nuts = BTreeSet::new();
                for announcement in announcements {
                    all_seen_nuts.extend(announcement.nuts);
                }

                Some((
                    mint_pubkey.clone(),
                    CashuAnnouncement {
                        mint_pubkey,
                        url: most_common_url,
                        nuts: all_seen_nuts,
                    },
                ))
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FedimintAnnouncement {
    pub federation_id: FederationId,
    pub invite_codes: BTreeSet<InviteCode>,
    pub modules: BTreeSet<String>,
}

impl FedimintAnnouncement {
    /// Merge & deduplicate a list of announcements such that the resulting set
    /// contains only one announcement per federation id, and all invite codes
    /// and modules seen in any announcement for a given federation.
    fn aggregate(announcements: Vec<Self>) -> BTreeMap<FederationId, Self> {
        let mut announcements_by_federation_id: BTreeMap<FederationId, Vec<Self>> = BTreeMap::new();
        for announcement in announcements {
            announcements_by_federation_id
                .entry(announcement.federation_id.clone())
                .or_default()
                .push(announcement);
        }

        announcements_by_federation_id
            .into_iter()
            .filter_map(|(federation_id, announcements)| {
                let all_seen_invite_codes = announcements
                    .iter()
                    .map(|a| a.invite_codes.clone())
                    .flatten()
                    .collect::<BTreeSet<_>>();

                let mut all_seen_modules = BTreeSet::new();
                for announcement in announcements {
                    all_seen_modules.extend(announcement.modules);
                }

                Some((
                    federation_id.clone(),
                    FedimintAnnouncement {
                        federation_id,
                        invite_codes: all_seen_invite_codes,
                        modules: all_seen_modules,
                    },
                ))
            })
            .collect()
    }
}

pub async fn discover_mints(
    network: Network,
) -> Result<
    (
        BTreeMap<String, CashuAnnouncement>,
        BTreeMap<FederationId, FedimintAnnouncement>,
    ),
    nostr_sdk::client::Error,
> {
    let network_strs = match network {
        // Note: NIP-87 specifies that "mainnet" should be used, but currently all
        // announcements on existing relays use "bitcoin" instead, so we filter for both.
        Network::Bitcoin => vec!["mainnet", "bitcoin"],
        Network::Testnet | Network::Testnet4 => vec!["testnet"],
        Network::Signet => vec!["signet"],
        Network::Regtest => vec!["regtest"],
        unknown_network => panic!("Unsupported network: {unknown_network}"),
    };

    // Create a new nostr client with an ephemeral keypair.
    // We're only going to read from relays, so the keypair
    // isn't ever actually used.
    let client = Client::new(Keys::generate());
    for relay in HARDCODED_RELAYS {
        client.add_relay(relay).await?;
    }
    client.connect().await;
    client.wait_for_connection(Duration::from_secs(10)).await;

    let nip87_announcement_filter = Filter::new()
        .kinds(vec![
            Kind::from_u16(NIP87_MINT_ANNOUNCEMENT_CASHU_KIND),
            Kind::from_u16(NIP87_MINT_ANNOUNCEMENT_FEDIMINT_KIND),
        ])
        .custom_tags(SingleLetterTag::lowercase(Alphabet::N), network_strs);

    let mut cashu_announcements = Vec::new();
    let mut fedimint_announcements = Vec::new();

    for event in client
        .fetch_events(nip87_announcement_filter, Duration::from_secs(10))
        .await?
    {
        if event.kind == Kind::from_u16(NIP87_MINT_ANNOUNCEMENT_CASHU_KIND) {
            if let Some(announcement) = parse_event_as_cashu_mint_announcement(event) {
                cashu_announcements.push(announcement);
            }
        } else if event.kind == Kind::from_u16(NIP87_MINT_ANNOUNCEMENT_FEDIMINT_KIND) {
            if let Some(announcement) = parse_event_as_fedimint_mint_announcement(event) {
                fedimint_announcements.push(announcement);
            }
        }
    }

    Ok((
        CashuAnnouncement::aggregate(cashu_announcements),
        FedimintAnnouncement::aggregate(fedimint_announcements),
    ))
}

fn parse_event_as_cashu_mint_announcement(event: Event) -> Option<CashuAnnouncement> {
    let mint_pubkey = event
        .tags
        .find(TagKind::SingleLetter(SingleLetterTag::lowercase(
            Alphabet::D,
        )))?
        .content()?
        .to_string();

    let url = event
        .tags
        .find(TagKind::SingleLetter(SingleLetterTag::lowercase(
            Alphabet::U,
        )))?
        .content()?
        .to_string();

    let nuts = event
        .tags
        .find(TagKind::Custom("nuts".into()))?
        .clone()
        .pop()?
        .split(',')
        .filter_map(|module| module.parse().ok())
        .collect();

    Some(CashuAnnouncement {
        mint_pubkey,
        url,
        nuts,
    })
}

fn parse_event_as_fedimint_mint_announcement(event: Event) -> Option<FedimintAnnouncement> {
    let federation_id = event
        .tags
        .find(TagKind::SingleLetter(SingleLetterTag::lowercase(
            Alphabet::D,
        )))
        .map(|tag| tag.content())?
        .map(|tag_content| FederationId::from_str(tag_content).ok())??;

    let invite_codes = event
        .tags
        .filter(TagKind::SingleLetter(SingleLetterTag::lowercase(
            Alphabet::U,
        )))
        .filter_map(|tag| tag.clone().pop())
        .filter_map(|code| InviteCode::from_str(&code).ok())
        .collect();

    let modules = event
        .tags
        .find(TagKind::Custom("modules".into()))?
        .clone()
        .pop()?
        .split(',')
        .map(|module| module.to_string())
        .collect();

    Some(FedimintAnnouncement {
        federation_id,
        invite_codes,
        modules,
    })
}

fn get_most_common_string<'a>(strings: &[&'a str]) -> Option<&'a str> {
    let mut counts = BTreeMap::new();
    let mut max_count = 0;
    let mut most_common = None;

    for string in strings {
        let count = counts.entry(string).or_insert(0);
        *count += 1;

        if *count > max_count {
            max_count = *count;
            most_common = Some(*string);
        }
    }

    most_common
}
