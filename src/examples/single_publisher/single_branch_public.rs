use iota_streams::{
    app::transport::tangle::client::Client,
    app_channels::api::tangle::{Address, Author, Bytes, ChannelType, Subscriber},
    core::{println, Result},
};

use crate::examples::{verify_messages, ALPH9};
use rand::Rng;
use core::str::FromStr;

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
    let mut author = Author::new(seed, ChannelType::SingleBranch, client.clone());

    // Create the channel with an announcement message. Make sure to save the resulting link somewhere,
    let announcement_link = author.send_announce().await?;
    // This link acts as a root for the channel itself
    let ann_link_string = announcement_link.to_string();
    println!(
        "Announcement Link: {}\nTangle Index: {:#}\n",
        ann_link_string, announcement_link.to_msg_index()
    );

    // Author will now send signed encrypted messages in a chain
    let msg_inputs = vec![
        "These", "Messages", "Will", "Be", "Masked", "And", "Sent", "In", "A", "Chain",
    ];

    let mut prev_msg_link = announcement_link;
    for input in &msg_inputs {
        let (msg_link, _seq_link) = author.send_signed_packet(
            &prev_msg_link,
            &Bytes::default(),
            &Bytes(input.as_bytes().to_vec()),
        ).await?;
        println!("Sent msg: {}", msg_link);
        prev_msg_link = msg_link;
    }

    // ------------------------------------------------------------------
    // In their own separate instances generate the subscriber(s) that will be attaching to the channel
    let mut subscriber = Subscriber::new("SubscriberA", client);

    // Generate an Address object from the provided announcement link string from the Author
    let ann_address = Address::from_str(&ann_link_string)?;

    // Receive the announcement message to start listening to the channel
    subscriber.receive_announcement(&ann_address).await?;

    let retrieved = subscriber.fetch_all_next_msgs().await;
    verify_messages(&msg_inputs, retrieved)?;

    Ok(())
}
