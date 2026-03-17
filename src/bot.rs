use nostr_sdk::prelude::*;
use tracing::{error, info, warn};

use crate::commands;
use crate::polymarket::gamma::GammaClient;

/// Run the bot event loop.
///
/// Subscribes to:
/// - NIP-17 Gift Wrap private DMs (Kind::GiftWrap where bot is the recipient)
/// - Kind 1 text notes that mention the bot's public key
///
/// Dispatches incoming messages to the command router and replies accordingly.
pub async fn run(client: &Client, gamma: &GammaClient) -> Result<()> {
    let keys = client.signer().await?;
    let pubkey = keys.get_public_key().await?;

    info!(
        pubkey_hex = %pubkey.to_hex(),
        pubkey_bech32 = %pubkey.to_bech32().unwrap_or_default(),
        "Setting up subscriptions for bot pubkey"
    );

    // Subscribe to GiftWrap (NIP-17 private DMs) - limit(0) skips history
    let dm_filter = Filter::new().pubkey(pubkey).kind(Kind::GiftWrap).limit(0);

    // Subscribe to text notes that mention the bot in 'p' tags
    let mention_filter = Filter::new()
        .kind(Kind::TextNote)
        .custom_tag(SingleLetterTag::lowercase(Alphabet::P), pubkey.to_hex())
        .limit(0);

    // Also subscribe to NIP-04 encrypted DMs as fallback (Kind 4)
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

    // Use handle_notifications pattern from the SDK examples
    client
        .handle_notifications(|notification| async {
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
                            info!("Event is GiftWrap (NIP-17) - processing...");
                            handle_gift_wrap(client, gamma, &event).await;
                        }
                        Kind::EncryptedDirectMessage => {
                            info!("Event is EncryptedDirectMessage (NIP-04) - processing...");
                            handle_nip04_dm(client, gamma, &event).await;
                        }
                        Kind::TextNote => {
                            handle_mention(client, gamma, &event).await;
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
            Ok(false) // false = continue loop
        })
        .await?;

    Ok(())
}

/// Handle a NIP-17 Gift Wrap private DM.
async fn handle_gift_wrap(client: &Client, gamma: &GammaClient, event: &Event) {
    info!("Attempting to unwrap gift wrap event...");

    match client.unwrap_gift_wrap(event).await {
        Ok(UnwrappedGift { rumor, sender }) => {
            info!(
                rumor_kind = ?rumor.kind,
                sender = %sender.to_hex(),
                "Successfully unwrapped gift wrap"
            );

            if rumor.kind == Kind::PrivateDirectMessage {
                info!(
                    sender = %sender.to_bech32().unwrap_or_default(),
                    message = %rumor.content,
                    "Received private DM"
                );

                let response = commands::handle_command(gamma, &rumor.content).await;

                info!("Sending DM reply...");
                // Send private message using client method
                if let Err(e) = client.send_private_msg(sender, response, []).await {
                    error!(error = %e, "Failed to send DM reply");
                } else {
                    info!("DM reply sent successfully");
                }
            } else {
                info!(
                    rumor_kind = ?rumor.kind,
                    "Unwrapped gift wrap but rumor is not a PrivateDirectMessage"
                );
            }
        }
        Err(e) => {
            warn!(error = %e, "Failed to decrypt gift wrap message");
        }
    }
}

/// Handle a NIP-04 encrypted direct message.
async fn handle_nip04_dm(client: &Client, gamma: &GammaClient, event: &Event) {
    info!(
        sender = %event.pubkey.to_bech32().unwrap_or_default(),
        "Attempting to decrypt NIP-04 DM..."
    );

    // Try to get the signer and decrypt
    match client.signer().await {
        Ok(signer) => {
            match signer.nip04_decrypt(&event.pubkey, &event.content).await {
                Ok(decrypted_content) => {
                    info!(
                        sender = %event.pubkey.to_bech32().unwrap_or_default(),
                        message = %decrypted_content,
                        "Received NIP-04 DM"
                    );

                    let response = commands::handle_command(gamma, &decrypted_content).await;

                    info!("Sending NIP-04 DM reply...");
                    // Use send_private_msg which handles NIP-17, or build a NIP-04 event manually
                    // For now, let's build a NIP-04 encrypted DM event
                    match signer.nip04_encrypt(&event.pubkey, &response).await {
                        Ok(encrypted) => {
                            let dm_event =
                                EventBuilder::new(Kind::EncryptedDirectMessage, encrypted)
                                    .tag(Tag::public_key(event.pubkey));
                            match client.send_event_builder(dm_event).await {
                                Ok(_) => info!("NIP-04 DM reply sent successfully"),
                                Err(e) => error!(error = %e, "Failed to send NIP-04 DM reply"),
                            }
                        }
                        Err(e) => error!(error = %e, "Failed to encrypt NIP-04 reply"),
                    }
                }
                Err(e) => {
                    warn!(error = %e, "Failed to decrypt NIP-04 DM");
                }
            }
        }
        Err(e) => {
            error!(error = %e, "Failed to get signer for NIP-04 decryption");
        }
    }
}

/// Handle a public text note that mentions the bot.
async fn handle_mention(client: &Client, gamma: &GammaClient, event: &Event) {
    let content = &event.content;
    let author = event.pubkey;

    info!(
        author = %author.to_bech32().unwrap_or_default(),
        message = %content,
        "Received public mention"
    );

    // Strip any nostr: prefixes (npub, nprofile, etc.) from the content
    // This handles cases where clients use different NIP-19 identifiers
    let clean_content = content
        .split_whitespace()
        .filter(|word| {
            // Filter out words that are nostr identifiers
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

    info!(
        cleaned_message = %clean_content,
        "Cleaned mention content"
    );

    let response = commands::handle_command(gamma, clean_content).await;

    // Reply as a public text note with proper threading tags
    let reply = EventBuilder::text_note(&response)
        .tag(Tag::public_key(author))
        .tag(Tag::event(event.id));

    if let Err(e) = client.send_event_builder(reply).await {
        error!(error = %e, "Failed to send public reply");
    }
}
