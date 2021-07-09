use iota_streams::{
    app::transport::tangle::client::Client,
    app_channels::api::tangle::{
        Address, Author, Bytes, MessageContent, PublicKey, Subscriber,
        UnwrappedMessage,
    },
    core::{println, Result},
};

use crate::examples::{verify_messages, ALPH9};
use rand::Rng;

/**
 * In this example, the Author will generate a new branch for each Subscriber, and each Subscriber
 * will only post/read from their individual branches
*/
pub fn example(node_url: &str) -> Result<()> {
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
    let mut author = Author::new(seed, "utf-8", 1024, true, client.clone());
    // Create the channel with an announcement message. Make sure to save the resulting link somewhere,
    let announcement_link = author.send_announce()?;
    // This link acts as a root for the channel itself
    let ann_link_string = announcement_link.to_string();
    println!(
        "Announcement Link: {}\nTangle Index: {}\n",
        ann_link_string, announcement_link
    );

    // ------------------------------------------------------------------
    // In their own separate instances generate the subscriber(s) that will be attaching to the channel

    let mut subscriber_a = Subscriber::new("SubscriberA", "utf-8", 1024, client.clone());
    let mut subscriber_b = Subscriber::new("SubscriberB", "utf-8", 1024, client.clone());
    let mut subscriber_c = Subscriber::new("SubscriberC", "utf-8", 1024, client);

    // Generate an Address object from the provided announcement link string from the Author
    let ann_link_split = ann_link_string.split(':').collect::<Vec<&str>>();
    let ann_address = Address::from_str(ann_link_split[0], ann_link_split[1])?;

    // Receive the announcement message to start listening to the channel
    subscriber_a.receive_announcement(&ann_address)?;
    subscriber_b.receive_announcement(&ann_address)?;
    subscriber_c.receive_announcement(&ann_address)?;

    // Subscribers send subscription messages linked to announcement message
    let subscribe_msg_a = subscriber_a.send_subscribe(&ann_address)?;
    let subscribe_msg_b = subscriber_b.send_subscribe(&ann_address)?;
    let subscribe_msg_c = subscriber_c.send_subscribe(&ann_address)?;

    // These are the subscription links that should be provided to the Author to complete subscription
    let sub_msg_a_str = subscribe_msg_a.to_string();
    let sub_msg_b_str = subscribe_msg_b.to_string();
    let sub_msg_c_str = subscribe_msg_c.to_string();

    println!(
        "Subscription msgs:\n\tSubscriber A: {}\n\tTangle Index: {}\n",
        sub_msg_a_str, subscribe_msg_a
    );
    println!(
        "\tSubscriber B: {}\n\tTangle Index: {}\n",
        sub_msg_b_str, subscribe_msg_b
    );
    println!(
        "\tSubscriber C: {}\n\tTangle Index: {}\n",
        sub_msg_c_str, subscribe_msg_c
    );

    // Fetch subscriber public keys (for use by author in issuing a keyload)
    let sub_a_pk = subscriber_a.get_pk().as_bytes();
    let sub_b_pk = subscriber_b.get_pk().as_bytes();
    let sub_c_pk = subscriber_c.get_pk().as_bytes();

    // We'll use this to sort messages on the retrieval end
    let pks = vec![
        PublicKey::from_bytes(sub_a_pk)?,
        PublicKey::from_bytes(sub_b_pk)?,
        PublicKey::from_bytes(sub_c_pk)?,
    ];
    // ----------------------------------------------------------------------
    // Get Address object from subscription message link provided by Subscriber A
    let sub_a_link_split = sub_msg_a_str.split(':').collect::<Vec<&str>>();
    let sub_a_address = Address::from_str(sub_a_link_split[0], sub_a_link_split[1])?;

    // Get Address object from subscription message link provided by SubscriberB
    let sub_b_link_split = sub_msg_b_str.split(':').collect::<Vec<&str>>();
    let sub_b_address = Address::from_str(sub_b_link_split[0], sub_b_link_split[1])?;

    // Get Address object from subscription message link provided by Subscriber C
    let sub_c_link_split = sub_msg_c_str.split(':').collect::<Vec<&str>>();
    let sub_c_address = Address::from_str(sub_c_link_split[0], sub_c_link_split[1])?;

    // Author processes subscription messages
    author.receive_subscribe(&sub_a_address)?;
    author.receive_subscribe(&sub_b_address)?;
    author.receive_subscribe(&sub_c_address)?;

    // Expectant users are now ready to be included in Keyload messages

    // Author sends keyload with the public key of Sub A (linked to announcement message) to generate
    // a new branch. This will return a tuple containing the message links. The first is the message
    // link itself, the second is a sequencing message.
    let (keyload_a_link, _seq_a_link) = author.send_keyload(
        &announcement_link,
        &[],
        &vec![PublicKey::from_bytes(sub_a_pk)?],
    )?;
    println!(
        "\nSent Keyload for Sub A: {}, seq: {}",
        keyload_a_link,
        _seq_a_link.unwrap()
    );

    // Author will send the second Keyload with the public key of Subscriber B (also linked to the
    // announcement message) to generate another new branch
    let (keyload_b_link, _seq_b_link) = author.send_keyload(
        &announcement_link,
        &[],
        &vec![PublicKey::from_bytes(sub_b_pk)?],
    )?;
    println!(
        "\nSent Keyload for Sub B: {}, seq: {}",
        keyload_b_link,
        _seq_b_link.unwrap()
    );

    // Author will send the third Keyload with the public key of Subscriber C (also linked to the
    // announcement message) to generate another new branch
    let (keyload_c_link, _seq_c_link) = author.send_keyload(
        &announcement_link,
        &[],
        &vec![PublicKey::from_bytes(sub_c_pk)?],
    )?;
    println!(
        "\nSent Keyload for Sub C: {}, seq: {}\n",
        keyload_c_link,
        _seq_c_link.unwrap()
    );

    // Before sending any messages, a publisher in a multi publisher channel should sync their state
    // to ensure they are up to date
    subscriber_a.sync_state();
    subscriber_b.sync_state();
    subscriber_c.sync_state();

    // Subscriber A will now send signed encrypted messages in a chain attached to Keyload A
    let msg_inputs_a = vec![
        "These",
        "Messages",
        "Will",
        "Be",
        "Masked",
        "And",
        "Sent",
        "By",
        "Subscriber",
        "A",
    ];

    let mut prev_msg_link = keyload_a_link;
    for input in &msg_inputs_a {
        let (msg_link, seq_link) = subscriber_a.send_signed_packet(
            &prev_msg_link,
            &Bytes::default(),
            &Bytes(input.as_bytes().to_vec()),
        )?;
        let seq_link = seq_link.unwrap();
        println!("Sent msg from Sub A: {}, seq: {}", msg_link, seq_link);
        prev_msg_link = msg_link;
    }

    // Subscriber B will now send signed encrypted messages in a chain attached to Keyload B
    let msg_inputs_b = vec![
        "These",
        "Messages",
        "Will",
        "Be",
        "Masked",
        "And",
        "Sent",
        "By",
        "Subscriber",
        "B",
    ];

    prev_msg_link = keyload_b_link;
    for input in &msg_inputs_b {
        let (msg_link, seq_link) = subscriber_b.send_signed_packet(
            &prev_msg_link,
            &Bytes::default(),
            &Bytes(input.as_bytes().to_vec()),
        )?;
        let seq_link = seq_link.unwrap();
        println!("Sent msg from Sub B: {}, seq: {}", msg_link, seq_link);
        prev_msg_link = msg_link;
    }

    // Lastly Subscriber C will now send signed encrypted messages in a chain attached to Keyload C
    let msg_inputs_c = vec![
        "These",
        "Messages",
        "Will",
        "Be",
        "Masked",
        "And",
        "Sent",
        "By",
        "Subscriber",
        "C",
    ];

    prev_msg_link = keyload_c_link;
    for input in &msg_inputs_c {
        let (msg_link, seq_link) = subscriber_c.send_signed_packet(
            &prev_msg_link,
            &Bytes::default(),
            &Bytes(input.as_bytes().to_vec()),
        )?;
        let seq_link = seq_link.unwrap();
        println!("Sent msg from Sub C: {}, seq: {}", msg_link, seq_link);
        prev_msg_link = msg_link;
    }

    // -----------------------------------------------------------------------------
    // Author can now fetch these messages
    let mut retrieved = author.fetch_all_next_msgs();
    println!("\nFound {} msgs", retrieved.len());
    let (retrieveda, retrievedb, retrievedc) = split_retrieved(&mut retrieved, pks);
    println!("\nVerifying message retrieval: Author");
    verify_messages(&msg_inputs_a, retrieveda)?;
    verify_messages(&msg_inputs_b, retrievedb)?;
    verify_messages(&msg_inputs_c, retrievedc)?;

    Ok(())
}

fn split_retrieved(
    retrieved: &mut Vec<UnwrappedMessage>,
    pks: Vec<PublicKey>,
) -> (
    Vec<UnwrappedMessage>,
    Vec<UnwrappedMessage>,
    Vec<UnwrappedMessage>,
) {
    let mut retrieved_msgs_a = Vec::new();
    let mut retrieved_msgs_b = Vec::new();
    let mut retrieved_msgs_c = Vec::new();

    // Sort messages by sender
    for _ in 0..retrieved.len() {
        let msg = retrieved.remove(0);
        let pk = match msg.body {
            MessageContent::SignedPacket {
                pk,
                public_payload: _,
                masked_payload: _,
            } => pk,
            _ => PublicKey::default(),
        };

        if pk == pks[0] {
            retrieved_msgs_a.push(msg);
        } else if pk == pks[1] {
            retrieved_msgs_b.push(msg);
        } else {
            retrieved_msgs_c.push(msg);
        }
    }

    (retrieved_msgs_a, retrieved_msgs_b, retrieved_msgs_c)
}
