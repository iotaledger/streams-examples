use iota_streams::{
    app::transport::tangle::client::Client,
    app_channels::api::tangle::{Address, Author, Bytes, ChannelType, Subscriber},
    core::{println, Result},
};

use crate::examples::{verify_messages, ALPH9};
use rand::Rng;
use core::str::FromStr;
use iota_streams::app_channels::api::tangle::MessageContent;

pub async fn example(node_url: &str) -> Result<()> {
    // Generate a unique seed for the author
    let seed: &str = &(0..81)
        .map(|_| {
            ALPH9
                .chars()
                .nth(rand::thread_rng().gen_range(0, 27))
                .unwrap()
        })
        .collect::<String>();

    // Create the Transport Client
    let client = Client::new_from_url(node_url);

    // Generate an Author
    let mut author = Author::new(seed, ChannelType::SingleDepth, client.clone());

    // Create the channel with an announcement message. Make sure to save the resulting link somewhere,
    let announcement_link = author.send_announce().await?;
    // This link acts as a root for the channel itself
    let ann_link_string = announcement_link.to_string();
    println!(
        "Announcement Link: {}\nTangle Index: {:#}\n",
        ann_link_string, announcement_link.to_msg_index()
    );

    // ------------------------------------------------------------------
    // In their own separate instances generate the subscriber(s) that will be attaching to the channel
    let mut subscriber_a = Subscriber::new("SubscriberA", client.clone());
    let mut subscriber_b = Subscriber::new("SubscriberB", client);

    // Generate an Address object from the provided announcement link string from the Author
    let ann_address = Address::from_str(&ann_link_string)?;

    // Receive the announcement message to start listening to the channel
    subscriber_a.receive_announcement(&ann_address).await?;
    subscriber_b.receive_announcement(&ann_address).await?;
    // ----------------------------------------------------------------------

    // Author will now send signed encrypted messages in a chain
    let msg_inputs = vec![
        "These", "Messages", "Will", "Be", "Masked", "And", "Sent", "In", "A", "Chain",
    ];

    // In a single depth implementation all messages will be anchored to the same message, and
    // messages can be retrieved by sequence number
    let anchor_msg_link = announcement_link;
    for input in &msg_inputs {
        let (msg_link, _seq_link) = author.send_signed_packet(
            &anchor_msg_link,
            &Bytes::default(),
            &Bytes(input.as_bytes().to_vec()),
        ).await?;
        println!("Sent msg: {}, tangle index: {:#}", msg_link, msg_link.to_msg_index());
    }

    // -----------------------------------------------------------------------------
    // Subscribers can now fetch these messages
    let mut retrieved = subscriber_a.fetch_all_next_msgs().await;
    verify_messages(&msg_inputs, retrieved)?;

    retrieved = subscriber_b.fetch_all_next_msgs().await;
    verify_messages(&msg_inputs, retrieved)?;

    // A message can be retreived by its sequence number and anchor message
    let msg_3 = subscriber_b.receive_msg_by_sequence_number(&anchor_msg_link, 2).await?;
    match msg_3.body {
        MessageContent::SignedPacket { pk: _, public_payload: _, masked_payload } => {
            assert_eq!(String::from_utf8(masked_payload.0)?, msg_inputs[2]);
            println!("3rd Message sent matches the message retrieved");
        }
        _ => panic!("Not a signed packet")
    }

    Ok(())
}
