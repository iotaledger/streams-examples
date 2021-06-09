use iota_streams::{
    app::transport::tangle::client::Client,
    app_channels::api::tangle::{
        Address, Author, Bytes, ChannelType, PublicKey, Subscriber, UnwrappedMessage,
    },
    core::{println, Result},
};

use crate::examples::{verify_messages, ALPH9};
use rand::Rng;

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
    let mut author = Author::new(seed, ChannelType::MultiBranch, client.clone());

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

    // This subscriber will subscribe traditionally
    let mut subscriber_a = Subscriber::new("SubscriberA", client.clone());
    // This subscriber will be added later in the channel
    let mut subscriber_b = Subscriber::new("SubscriberB", client);

    // Generate an Address object from the provided announcement link string from the Author
    let ann_link_split = ann_link_string.split(':').collect::<Vec<&str>>();
    let ann_address = Address::from_str(ann_link_split[0], ann_link_split[1])?;

    // Receive the announcement message to start listening to the channel
    subscriber_a.receive_announcement(&ann_address)?;
    subscriber_b.receive_announcement(&ann_address)?;

    // Subs A and B send subscription messages linked to announcement message
    let subscribe_msg_a = subscriber_a.send_subscribe(&ann_address)?;
    let subscribe_msg_b = subscriber_b.send_subscribe(&ann_address)?;

    let sub_a_pk = subscriber_a.get_pk().as_bytes();
    let sub_b_pk = subscriber_b.get_pk().as_bytes();

    // These are the subscription links that should be provided to the Author to complete
    // subscription for users A and B
    let sub_msg_a_str = subscribe_msg_a.to_string();
    let sub_msg_b_str = subscribe_msg_b.to_string();

    println!(
        "Subscription msgs:\n\tSubscriber A: {}\n\tTangle Index: {}\n\tSubscriber B: {}\n\tTangle Index: {}\n",
        sub_msg_a_str, subscribe_msg_a, sub_msg_b_str, subscribe_msg_b,
    );
    // ----------------------------------------------------------------------

    // Get Address object from subscription message link provided by Subscriber A
    let sub_a_link_split = sub_msg_a_str.split(':').collect::<Vec<&str>>();
    let sub_a_address = Address::from_str(sub_a_link_split[0], sub_a_link_split[1])?;

    // Get Address object from subscription message link provided by Subscriber B
    let sub_b_link_split = sub_msg_b_str.split(':').collect::<Vec<&str>>();
    let sub_b_address = Address::from_str(sub_b_link_split[0], sub_b_link_split[1])?;

    // Author processes subscribers A and B
    author.receive_subscribe(&sub_a_address)?;
    author.receive_subscribe(&sub_b_address)?;

    // Expectant users are now ready to be included in Keyload messages

    // Author sends keyload with the public key of Sub A (linked to announcement message) to generate
    // a new branch. This will return a tuple containing the message links. The first is the message
    // link itself, the second is an optional sequencing message.
    let (_keyload_a_link, seq_a_link) = author.send_keyload(
        &announcement_link,
        &[],
        &vec![PublicKey::from_bytes(sub_a_pk)?],
    )?;
    println!(
        "\nSent Keyload for Sub A: {}, seq: {}\n",
        _keyload_a_link,
        seq_a_link.as_ref().unwrap()
    );

    // Author will now send signed encrypted messages to Sub A in a chain attached to Keyload A
    let msg_inputs_a = vec![
        "These",
        "Messages",
        "Will",
        "Be",
        "Masked",
        "And",
        "Only",
        "Readable",
        "By",
        "Subscriber",
        "A",
    ];

    let mut prev_msg_link = _keyload_a_link;
    let mut seq_msg_link = seq_a_link.unwrap();
    for input in &msg_inputs_a {
        let (msg_link, seq_link) = author.send_signed_packet(
            &prev_msg_link,
            &Bytes::default(),
            &Bytes(input.as_bytes().to_vec()),
        )?;
        let seq_link = seq_link.unwrap();
        println!("Sent msg for Sub A: {}, seq: {}", msg_link, seq_link);
        prev_msg_link = msg_link;
        seq_msg_link = seq_link;
    }

    // Author will send the second Keyload with the Public Key of Subscriber B attached to the
    // sequence message of the previous message.
    //
    // ** In order to allow users to access the message without having permission for the previous
    // messages, the keyload can be attached to the sequence message link, since the sequence message
    // link is stored in state regardless of user access to the referenced message.
    let (_keyload_b_link, seq_b_link) =
        author.send_keyload(&seq_msg_link, &[], &vec![PublicKey::from_bytes(sub_b_pk)?])?;

    println!(
        "\nSent Keyload granting Sub B Forward Access, while revoking Sub A: {}, seq: {}\n",
        _keyload_b_link,
        seq_b_link.as_ref().unwrap()
    );

    // Author will now send signed encrypted messages to Sub B in a chain attached to Keyload B
    let msg_inputs_b = vec![
        "These",
        "Messages",
        "Will",
        "Be",
        "Masked",
        "And",
        "Only",
        "Readable",
        "By",
        "Subscriber",
        "B",
    ];

    prev_msg_link = _keyload_b_link;
    seq_msg_link = seq_b_link.unwrap();
    for input in &msg_inputs_b {
        let (msg_link, seq_link) = author.send_signed_packet(
            &prev_msg_link,
            &Bytes::default(),
            &Bytes(input.as_bytes().to_vec()),
        )?;
        let seq_link = seq_link.unwrap();
        println!("Sent msg for Sub B: {}, seq: {}", msg_link, seq_link);
        prev_msg_link = msg_link;
        seq_msg_link = seq_link;
    }

    // Author will send the third Keyload with the Public Key of Subscriber A again attached to the
    // sequence message of the previous message.
    //
    // ** In order to allow users to access the message without having permission for the previous
    // messages, the keyload can be attached to the sequence message link, since the sequence message
    // link is stored in state regardless of user access to the referenced message.
    let (_keyload_c_link, seq_c_link) =
        author.send_keyload(&seq_msg_link, &[], &vec![PublicKey::from_bytes(sub_a_pk)?])?;

    println!(
        "\nSent Keyload granting Sub A Forward Access again, while revoking Sub B: {}, seq: {}\n",
        _keyload_c_link,
        seq_c_link.as_ref().unwrap()
    );

    // Author will send signed encrypted messages to Sub A again in a chain attached to Keyload C
    let msg_inputs_c = vec![
        "These",
        "Messages",
        "Will",
        "Be",
        "Masked",
        "And",
        "Only",
        "Readable",
        "By",
        "Subscriber",
        "A",
        "Again",
    ];

    prev_msg_link = _keyload_c_link;
    for input in &msg_inputs_c {
        let (msg_link, seq_link) = author.send_signed_packet(
            &prev_msg_link,
            &Bytes::default(),
            &Bytes(input.as_bytes().to_vec()),
        )?;
        let seq_link = seq_link.unwrap();
        println!("Sent msg for Sub A again: {}, seq: {}", msg_link, seq_link);
        prev_msg_link = msg_link;
    }

    // -----------------------------------------------------------------------------
    // Subscribers can now fetch these messages
    let mut retrieved = subscriber_a.fetch_all_next_msgs();
    let (retrieveda, retrievedb, retrieveda2) = split_retrieved(
        &mut retrieved,
        msg_inputs_a.len(),
        msg_inputs_b.len(),
        msg_inputs_c.len(),
    );
    println!("\nVerifying message retrieval: SubscriberA");
    verify_messages(&msg_inputs_a, retrieveda)?;
    verify_messages(&[], retrievedb)?;
    verify_messages(&msg_inputs_c, retrieveda2)?;

    retrieved = subscriber_b.fetch_all_next_msgs();
    let (retrieveda, retrievedb, retrieveda2) = split_retrieved(
        &mut retrieved,
        msg_inputs_a.len(),
        msg_inputs_b.len(),
        msg_inputs_c.len(),
    );
    println!("\nVerifying message retrieval: SubscriberB");
    verify_messages(&[], retrieveda)?;
    verify_messages(&msg_inputs_b, retrievedb)?;
    verify_messages(&[], retrieveda2)?;

    Ok(())
}

fn split_retrieved(
    retrieved: &mut Vec<UnwrappedMessage>,
    len1: usize,
    len2: usize,
    len3: usize,
) -> (
    Vec<UnwrappedMessage>,
    Vec<UnwrappedMessage>,
    Vec<UnwrappedMessage>,
) {
    let mut retrieved_msgs_a = Vec::new();
    let mut retrieved_msgs_b = Vec::new();
    let mut retrieved_msgs_a2 = Vec::new();

    // Keyload A
    retrieved.remove(0);

    for _ in 0..len1 {
        // Messages for sub A
        retrieved_msgs_a.push(retrieved.remove(0));
    }

    // Keyload B
    retrieved.remove(0);

    for _ in 0..len2 {
        // Messages for sub B
        retrieved_msgs_b.push(retrieved.remove(0));
    }

    // Keyload C
    retrieved.remove(0);

    for _ in 0..len3 {
        // Messages for sub A again
        retrieved_msgs_a2.push(retrieved.remove(0));
    }

    (retrieved_msgs_a, retrieved_msgs_b, retrieved_msgs_a2)
}
