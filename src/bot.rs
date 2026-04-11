use std::sync::Arc;

use nostr_sdk::prelude::*;
use tracing::{error, info, warn};

use crate::alerts::manager::AlertManager;
use crate::alerts::model::DeliveryChannel;
use crate::commands;
use crate::polymarket::data::DataClient;
use crate::polymarket::gamma::GammaClient;

pub async fn run(
    client: Arc<Client>,
    gamma: Arc<GammaClient>,
    data: Arc<DataClient>,
    alert_manager: Arc<AlertManager>,
) -> Result<()> {
    let keys = client.signer().await?;
    let pubkey = keys.get_public_key().await?;

    info!(
        pubkey_hex = %pubkey.to_hex(),
        pubkey_bech32 = %pubkey.to_bech32().unwrap_or_default(),
        "Setting up subscriptions for bot pubkey"
    );

    let dm_filter = Filter::new().pubkey(pubkey).kind(Kind::GiftWrap).limit(0);
    let mention_filter = Filter::new()
        .kind(Kind::TextNote)
        .custom_tag(SingleLetterTag::lowercase(Alphabet::P), pubkey.to_hex())
        .limit(0);
    let nip04_dm_filter = Filter::new()
        .pubkey(pubkey)
        .kind(Kind::EncryptedDirectMessage)
        .limit(0);

    info!("Subscribing to GiftWrap (NIP-17) DMs...");
    client.subscribe(dm_filter, None).await?;
    info!("Subscribing to EncryptedDirectMessage (NIP-04) DMs...");
    client.subscribe(nip04_dm_filter, None).await?;
    info!("Subscribing to text note mentions...");
    client.subscribe(mention_filter, None).await?;

    info!("Bot is listening for messages...");

    client
        .handle_notifications(|notification| {
            let client = client.clone();
            let gamma = gamma.clone();
            let data = data.clone();
            let alert_manager = alert_manager.clone();
            async move {
                match notification {
                    RelayPoolNotification::Event { event, .. } => {
                        info!(
                            event_id = %event.id,
                            event_kind = ?event.kind,
                            event_pubkey = %event.pubkey.to_hex(),
                            "Received event"
                        );
                        match event.kind {
                            Kind::GiftWrap => {
                                handle_gift_wrap(client, gamma, data, alert_manager, &event).await;
                            }
                            Kind::EncryptedDirectMessage => {
                                handle_nip04_dm(client, gamma, data, alert_manager, &event).await;
                            }
                            Kind::TextNote => {
                                handle_mention(client, gamma, data, alert_manager, &event).await;
                            }
                            _ => info!(kind = ?event.kind, "Received unsupported event kind"),
                        }
                    }
                    RelayPoolNotification::Message { relay_url, message } => {
                        info!(relay = %relay_url, message = ?message, "Received relay message");
                    }
                    _ => {
                        info!("Received other notification type");
                    }
                }
                Ok(false)
            }
        })
        .await?;

    Ok(())
}

async fn handle_gift_wrap(
    client: Arc<Client>,
    gamma: Arc<GammaClient>,
    data: Arc<DataClient>,
    alert_manager: Arc<AlertManager>,
    event: &Event,
) {
    info!("Attempting to unwrap gift wrap event...");

    match client.unwrap_gift_wrap(event).await {
        Ok(UnwrappedGift { rumor, sender }) => {
            if rumor.kind == Kind::PrivateDirectMessage {
                info!(
                    sender = %sender.to_bech32().unwrap_or_default(),
                    message = %rumor.content,
                    "Received private DM"
                );

                let response = commands::handle_command(
                    &gamma,
                    &data,
                    alert_manager,
                    Some(sender.to_hex()),
                    DeliveryChannel::Nip17,
                    &rumor.content,
                )
                .await;

                if let Err(e) = client.send_private_msg(sender, response, []).await {
                    error!(error = %e, "Failed to send DM reply");
                }
            }
        }
        Err(e) => {
            warn!(error = %e, "Failed to decrypt gift wrap message");
        }
    }
}

async fn handle_nip04_dm(
    client: Arc<Client>,
    gamma: Arc<GammaClient>,
    data: Arc<DataClient>,
    alert_manager: Arc<AlertManager>,
    event: &Event,
) {
    match client.signer().await {
        Ok(signer) => match signer.nip04_decrypt(&event.pubkey, &event.content).await {
            Ok(decrypted_content) => {
                let response = commands::handle_command(
                    &gamma,
                    &data,
                    alert_manager,
                    Some(event.pubkey.to_hex()),
                    DeliveryChannel::Nip04,
                    &decrypted_content,
                )
                .await;

                match signer.nip04_encrypt(&event.pubkey, &response).await {
                    Ok(encrypted) => {
                        let dm_event = EventBuilder::new(Kind::EncryptedDirectMessage, encrypted)
                            .tag(Tag::public_key(event.pubkey));
                        if let Err(e) = client.send_event_builder(dm_event).await {
                            error!(error = %e, "Failed to send NIP-04 DM reply");
                        }
                    }
                    Err(e) => error!(error = %e, "Failed to encrypt NIP-04 reply"),
                }
            }
            Err(e) => {
                warn!(error = %e, "Failed to decrypt NIP-04 DM");
            }
        },
        Err(e) => {
            error!(error = %e, "Failed to get signer for NIP-04 decryption");
        }
    }
}

async fn handle_mention(
    client: Arc<Client>,
    gamma: Arc<GammaClient>,
    data: Arc<DataClient>,
    alert_manager: Arc<AlertManager>,
    event: &Event,
) {
    let content = &event.content;
    let author = event.pubkey;

    let clean_content = content
        .split_whitespace()
        .filter(|word| {
            !(word.starts_with("nostr:npub")
                || word.starts_with("nostr:nprofile")
                || word.starts_with("nostr:note")
                || word.starts_with("npub1")
                || word.starts_with("nprofile1")
                || word.starts_with("note1"))
        })
        .collect::<Vec<_>>()
        .join(" ");

    let clean_content = clean_content.trim();

    let response = commands::handle_command(
        &gamma,
        &data,
        alert_manager,
        Some(author.to_hex()),
        DeliveryChannel::Nip17,
        clean_content,
    )
    .await;

    let reply = EventBuilder::text_note(&response)
        .tag(Tag::public_key(author))
        .tag(Tag::event(event.id));

    if let Err(e) = client.send_event_builder(reply).await {
        error!(error = %e, "Failed to send public reply");
    }
}
