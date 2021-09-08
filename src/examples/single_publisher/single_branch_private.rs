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

    // ------------------------------------------------------------------
    // In their own separate instances generate the subscriber(s) that will be attaching to the channel
    let mut subscriber_a = Subscriber::new("SubscriberA", client.clone());
    let mut subscriber_b = Subscriber::new("SubscriberB", client);

    // Generate an Address object from the provided announcement link string from the Author
    let ann_address = Address::from_str(&ann_link_string)?;

    // Receive the announcement message to start listening to the channel
    subscriber_a.receive_announcement(&ann_address).await?;
    subscriber_b.receive_announcement(&ann_address).await?;

    // Send subscription messages linked to announcement message
    let subscribe_msg_a = subscriber_a.send_subscribe(&ann_address).await?;
    let subscribe_msg_b = subscriber_b.send_subscribe(&ann_address).await?;

    // These are the subscription links that should be provided to the Author to complete subscription
    let sub_msg_a_str = subscribe_msg_a.to_string();
    let sub_msg_b_str = subscribe_msg_b.to_string();

    println!("Subscription msgs:\n\tSubscriber A: {}\n\tTangle Index: {:#}\n\tSubscriber B: {}\n\tTangle Index: {:#}\n",
        sub_msg_a_str, subscribe_msg_a.to_msg_index(), sub_msg_b_str, subscribe_msg_b.to_msg_index());
    // ----------------------------------------------------------------------

    // Get Address objects from subscription message links provided by expectant subscribers
    let sub_a_address = Address::from_str(&sub_msg_a_str)?;
    let sub_b_address = Address::from_str(&sub_msg_b_str)?;

    // Author processes subscribers
    author.receive_subscribe(&sub_a_address).await?;
    author.receive_subscribe(&sub_b_address).await?;

    // Expectant users are now subscribed and can be included in a Keyload message

    // Author sends keyload with Subs A and B included (linked to announcement message). This will
    // return a tuple containing the message links. The first is the message link itself, the second
    // is an optional sequencing message.
    // ** In single branch implementations, sequencing messages are not sent and can be ignored
    let (keyload_link, _seq) = author.send_keyload_for_everyone(&announcement_link).await?;

    // Author will now send signed encrypted messages in a chain
    let msg_inputs = vec![
        "These", "Messages", "Will", "Be", "Masked", "And", "Sent", "In", "A", "Chain",
    ];

    let mut prev_msg_link = keyload_link;
    for input in &msg_inputs {
        let (msg_link, _seq_link) = author.send_signed_packet(
            &prev_msg_link,
            &Bytes::default(),
            &Bytes(input.as_bytes().to_vec()),
        ).await?;
        println!("Sent msg: {}, {}", msg_link, msg_link.to_msg_index());
        prev_msg_link = msg_link;
    }

    // -----------------------------------------------------------------------------
    // Subscribers can now fetch these messages
    let mut retrieved = subscriber_a.fetch_all_next_msgs().await;
    verify_messages(&msg_inputs, retrieved)?;

    retrieved = subscriber_b.fetch_all_next_msgs().await;
    verify_messages(&msg_inputs, retrieved)?;

    Ok(())
}
