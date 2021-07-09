use iota_streams::{
    app::transport::tangle::client::Client,
    app_channels::api::{
        pskid_from_psk,
        psk_from_seed,
        tangle::{
            Address, Author, Bytes, PublicKey, Subscriber, UnwrappedMessage,
        }
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
    let mut author = Author::new(seed, "utf-8", 1024, true, client.clone());
    // Create the channel with an announcement message. Make sure to save the resulting link somewhere,
    let announcement_link = author.send_announce()?;
    // This link acts as a root for the channel itself
    let ann_link_string = announcement_link.to_string();
    println!(
        "Announcement Link: {}\nTangle Index: {}\n",
        ann_link_string, announcement_link
    );

    // Generate a key to be used as a Pre Shared Key
    let key = rand::thread_rng().gen::<[u8; 32]>();

    // Author will now store a PSK to be used by Subscriber B. This will return a PskId (first half
    // of key for usage in keyload generation)
    let psk = psk_from_seed(&key);
    let pskid = pskid_from_psk(&psk);
    author.store_psk(pskid, psk);

    // ------------------------------------------------------------------
    // In their own separate instances generate the subscriber(s) that will be attaching to the channel

    // This subscriber will subscribe traditionally
    let mut subscriber_a = Subscriber::new("SubscriberA", "utf-8", 1024, client.clone());
    // This subscriber will use a PSK
    let mut subscriber_b = Subscriber::new("SubscriberB", "utf-8", 1024, client.clone());
    // This subscriber will not subscribe at all
    let mut subscriber_c = Subscriber::new("SubscriberC", "utf-8", 1024, client);

    // Generate an Address object from the provided announcement link string from the Author
    let ann_link_split = ann_link_string.split(':').collect::<Vec<&str>>();
    let ann_address = Address::from_str(ann_link_split[0], ann_link_split[1])?;

    // Receive the announcement message to start listening to the channel
    subscriber_a.receive_announcement(&ann_address)?;
    subscriber_b.receive_announcement(&ann_address)?;
    subscriber_c.receive_announcement(&ann_address)?;

    // Sub A sends subscription message linked to announcement message
    let subscribe_msg_a = subscriber_a.send_subscribe(&ann_address)?;

    // Fetch sub A public key (for use by author in issuing a keyload)
    let sub_a_pk = subscriber_a.get_pk().as_bytes();

    // Sub B stores PSK shared by Author
    let psk = psk_from_seed(&key);
    let pskid = pskid_from_psk(&psk);
    subscriber_b.store_psk(pskid, psk);

    // This is the subscription link that should be provided to the Author to complete subscription
    // for user A
    let sub_msg_a_str = subscribe_msg_a.to_string();

    println!(
        "Subscription msg:\n\tSubscriber A: {}\n\tTangle Index: {}\n",
        sub_msg_a_str, subscribe_msg_a
    );
    // ----------------------------------------------------------------------

    // Get Address object from subscription message link provided by Subscriber A
    let sub_a_link_split = sub_msg_a_str.split(':').collect::<Vec<&str>>();
    let sub_a_address = Address::from_str(sub_a_link_split[0], sub_a_link_split[1])?;

    // Author processes subscriber A
    author.receive_subscribe(&sub_a_address)?;

    // Expectant users are now ready to be included in Keyload messages

    // Author sends keyload with the public key of Sub A (linked to announcement message) to generate
    // a new branch. This will return a tuple containing the message links. The first is the message
    // link itself, the second is an optional sequencing message.
    // ** In multi branch implementations, sequencing messages are sent to act as indexing references
    // for data location within the channel tree
    let (_keyload_a_link, seq_a_link) = author.send_keyload(
        &announcement_link,
        &[],
        &vec![PublicKey::from_bytes(sub_a_pk)?],
    )?;
    println!(
        "\nSent Keyload for Sub A: {}, seq: {}",
        _keyload_a_link,
        seq_a_link.as_ref().unwrap()
    );

    // Author will send the second Keyload with the PSK shared with Subscriber B (also linked to the
    // announcement message) to generate another new branch
    let (_keyload_b_link, seq_b_link) =
        author.send_keyload(&announcement_link, &[pskid], &vec![])?;
    println!(
        "\nSent Keyload for Sub B: {}, seq: {}",
        _keyload_b_link,
        seq_b_link.as_ref().unwrap()
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
    for input in &msg_inputs_a {
        let (msg_link, seq_link) = author.send_signed_packet(
            &prev_msg_link,
            &Bytes::default(),
            &Bytes(input.as_bytes().to_vec()),
        )?;
        let seq_link = seq_link.unwrap();
        println!("Sent msg for Sub A: {}, seq: {}", msg_link, seq_link);
        prev_msg_link = msg_link;
    }

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

    let mut prev_msg_link = _keyload_b_link;
    for input in &msg_inputs_b {
        let (msg_link, seq_link) = author.send_signed_packet(
            &prev_msg_link,
            &Bytes::default(),
            &Bytes(input.as_bytes().to_vec()),
        )?;
        let seq_link = seq_link.unwrap();
        println!("Sent msg for Sub B: {}, seq: {}", msg_link, seq_link);
        prev_msg_link = msg_link;
    }

    // Lastly the Author will now send signed encrypted messages in a public chain readable by anyone (Subscriber C)
    let msg_inputs_all = vec![
        "These", "Messages", "Will", "Be", "Masked", "And", "Readable", "By", "Anyone",
    ];

    let mut prev_msg_link = announcement_link;
    for input in &msg_inputs_all {
        let (msg_link, seq_link) = author.send_signed_packet(
            &prev_msg_link,
            &Bytes::default(),
            &Bytes(input.as_bytes().to_vec()),
        )?;
        let seq_link = seq_link.unwrap();
        println!("Sent msg for Anyone: {}, seq: {}", msg_link, seq_link);
        prev_msg_link = msg_link;
    }

    // -----------------------------------------------------------------------------
    // Subscribers can now fetch these messages
    let mut retrieved = subscriber_a.fetch_all_next_msgs();
    let (retrieveda, retrievedb, retrieved_all) =
        split_retrieved(&mut retrieved, msg_inputs_a.len(), msg_inputs_b.len());
    println!("\nVerifying message retrieval: SubscriberA");
    verify_messages(&msg_inputs_a, retrieveda)?;
    verify_messages(&[], retrievedb)?;
    verify_messages(&msg_inputs_all, retrieved_all)?;

    retrieved = subscriber_b.fetch_all_next_msgs();
    let (retrieveda, retrievedb, retrieved_all) =
        split_retrieved(&mut retrieved, msg_inputs_a.len(), msg_inputs_b.len());
    println!("\nVerifying message retrieval: SubscriberB");
    verify_messages(&[], retrieveda)?;
    verify_messages(&msg_inputs_b, retrievedb)?;
    verify_messages(&msg_inputs_all, retrieved_all)?;

    retrieved = subscriber_c.fetch_all_next_msgs();
    println!("\nVerifying message retrieval: SubscriberC");
    verify_messages(&msg_inputs_all, retrieved)?;

    Ok(())
}

fn split_retrieved(
    retrieved: &mut Vec<UnwrappedMessage>,
    len1: usize,
    len2: usize,
) -> (
    Vec<UnwrappedMessage>,
    Vec<UnwrappedMessage>,
    Vec<UnwrappedMessage>,
) {
    let mut retrieved_msgs_a = Vec::new();
    let mut retrieved_msgs_b = Vec::new();
    let mut retrieved_msgs_all = Vec::new();
    for _ in 0..2 {
        // Keyloads
        retrieved.remove(0);
    }

    for _ in 0..len1 {
        // Messages for sub A
        retrieved_msgs_a.push(retrieved.remove(0));
    }

    for _ in 0..len2 {
        // Messages for sub B
        retrieved_msgs_b.push(retrieved.remove(0));
    }

    for _ in 0..retrieved.len() {
        // Messages for anyone
        retrieved_msgs_all.push(retrieved.remove(0));
    }

    (retrieved_msgs_a, retrieved_msgs_b, retrieved_msgs_all)
}
