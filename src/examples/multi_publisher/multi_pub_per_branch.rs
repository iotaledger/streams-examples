use iota_streams::{
    app::transport::tangle::client::Client,
    app_channels::api::tangle::{
        Address, Author, Bytes, ChannelType, MessageContent, PublicKey, Subscriber,
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

    let mut subscriber_a = Subscriber::new("SubscriberA", client.clone());
    let mut subscriber_b = Subscriber::new("SubscriberB", client.clone());
    let mut subscriber_c = Subscriber::new("SubscriberC", client.clone());
    let mut subscriber_d = Subscriber::new("SubscriberD", client);

    // Generate an Address object from the provided announcement link string from the Author
    let ann_link_split = ann_link_string.split(':').collect::<Vec<&str>>();
    let ann_address = Address::from_str(ann_link_split[0], ann_link_split[1])?;

    // Receive the announcement message to start listening to the channel
    subscriber_a.receive_announcement(&ann_address)?;
    subscriber_b.receive_announcement(&ann_address)?;
    subscriber_c.receive_announcement(&ann_address)?;
    subscriber_d.receive_announcement(&ann_address)?;

    // Subscribers send subscription messages linked to announcement message
    let subscribe_msg_a = subscriber_a.send_subscribe(&ann_address)?;
    let subscribe_msg_b = subscriber_b.send_subscribe(&ann_address)?;
    let subscribe_msg_c = subscriber_c.send_subscribe(&ann_address)?;
    let subscribe_msg_d = subscriber_d.send_subscribe(&ann_address)?;

    // These are the subscription links that should be provided to the Author to complete subscription
    let sub_msg_a_str = subscribe_msg_a.to_string();
    let sub_msg_b_str = subscribe_msg_b.to_string();
    let sub_msg_c_str = subscribe_msg_c.to_string();
    let sub_msg_d_str = subscribe_msg_d.to_string();

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
    println!(
        "\tSubscriber D: {}\n\tTangle Index: {}\n",
        sub_msg_d_str, subscribe_msg_d
    );

    // Fetch subscriber public keys (for use by author in issuing a keyload)
    let sub_a_pk = subscriber_a.get_pk().as_bytes();
    let sub_b_pk = subscriber_b.get_pk().as_bytes();
    let sub_c_pk = subscriber_c.get_pk().as_bytes();
    let sub_d_pk = subscriber_d.get_pk().as_bytes();

    // We'll use this to sort messages on the retrieval end
    let pks = vec![
        PublicKey::from_bytes(sub_a_pk)?,
        PublicKey::from_bytes(sub_b_pk)?,
        PublicKey::from_bytes(sub_c_pk)?,
        PublicKey::from_bytes(sub_d_pk)?,
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

    // Get Address object from subscription message link provided by Subscriber C
    let sub_d_link_split = sub_msg_d_str.split(':').collect::<Vec<&str>>();
    let sub_d_address = Address::from_str(sub_d_link_split[0], sub_d_link_split[1])?;

    // Author processes subscription messages
    author.receive_subscribe(&sub_a_address)?;
    author.receive_subscribe(&sub_b_address)?;
    author.receive_subscribe(&sub_c_address)?;
    author.receive_subscribe(&sub_d_address)?;

    // Expectant users are now ready to be included in Keyload messages

    // Author sends keyload with the public keys of Sub A and B (linked to announcement message) to
    // generate a new branch. This will return a tuple containing the message links. The first is
    // the message link itself, the second is the sequencing message link.
    let (keyload_a_link, _seq_a_link) =
        author.send_keyload(&announcement_link, &[], &vec![pks[0], pks[1]])?;
    println!(
        "\nSent Keyload for Sub A and B: {}, seq: {}",
        keyload_a_link,
        _seq_a_link.unwrap()
    );

    // Author will send the second Keyload with the public key of Subscribers C and D (also linked
    // to the announcement message) to generate another new branch
    let (keyload_b_link, _seq_b_link) =
        author.send_keyload(&announcement_link, &[], &vec![pks[2], pks[3]])?;
    println!(
        "\nSent Keyload for Sub C and D: {}, seq: {}\n",
        keyload_b_link,
        _seq_b_link.unwrap()
    );

    // Subscribers A and B will now send encrypted messages in an alternating chain attached to Keyload A
    let msg_inputs_a = vec![
        "These",
        "Messages",
        "Will",
        "Be",
        "Sent",
        "By",
        "Susbscriber",
        "A",
    ];
    let msg_inputs_b = vec![
        "These",
        "Messages",
        "Will",
        "Be",
        "Sent",
        "By",
        "Susbscriber",
        "B",
    ];

    let mut prev_msg_link = keyload_a_link;
    for i in 0..msg_inputs_a.len() {
        // ***********************  IMPORTANT  ****************************************
        // Before sending any messages, a publisher in a multi publisher channel should sync their state
        // to ensure they are up to date
        subscriber_a.sync_state();

        // Sub A Sends
        let (msg_link, seq_link) = subscriber_a.send_signed_packet(
            &prev_msg_link,
            &Bytes::default(),
            &Bytes(msg_inputs_a[i].as_bytes().to_vec()),
        )?;
        let seq_link = seq_link.unwrap();
        println!("Sent msg from Sub A: {}, seq: {}", msg_link, seq_link);
        prev_msg_link = msg_link;

        // Sub B Sends
        subscriber_b.sync_state();
        let (msg_link, seq_link) = subscriber_b.send_signed_packet(
            &prev_msg_link,
            &Bytes::default(),
            &Bytes(msg_inputs_b[i].as_bytes().to_vec()),
        )?;
        let seq_link = seq_link.unwrap();
        println!("Sent msg from Sub B: {}, seq: {}", msg_link, seq_link);
        prev_msg_link = msg_link;
    }

    // Subscribers C and D will now send encrypted messages in an alternating chain attached to Keyload B
    let msg_inputs_c = vec![
        "These",
        "Messages",
        "Will",
        "Be",
        "Sent",
        "By",
        "Subscriber",
        "C",
    ];
    let msg_inputs_d = vec![
        "These",
        "Messages",
        "Will",
        "Be",
        "Sent",
        "By",
        "Subscriber",
        "D",
    ];

    prev_msg_link = keyload_b_link;
    for i in 0..msg_inputs_c.len() {
        // Sub C Sends
        subscriber_c.sync_state();
        let (msg_link, seq_link) = subscriber_c.send_signed_packet(
            &prev_msg_link,
            &Bytes::default(),
            &Bytes(msg_inputs_c[i].as_bytes().to_vec()),
        )?;
        let seq_link = seq_link.unwrap();
        println!("Sent msg from Sub C: {}, seq: {}", msg_link, seq_link);
        prev_msg_link = msg_link;

        // Sub D Sends
        subscriber_d.sync_state();
        let (msg_link, seq_link) = subscriber_d.send_signed_packet(
            &prev_msg_link,
            &Bytes::default(),
            &Bytes(msg_inputs_d[i].as_bytes().to_vec()),
        )?;
        let seq_link = seq_link.unwrap();
        println!("Sent msg from Sub D: {}, seq: {}", msg_link, seq_link);
        prev_msg_link = msg_link;
    }

    // -----------------------------------------------------------------------------
    // Author can now fetch these messages
    let mut retrieved = author.fetch_all_next_msgs();
    println!("\nAuthor found {} messages", retrieved.len());
    let mut retrieved_lists = split_retrieved(&mut retrieved, pks);
    println!("\nVerifying message retrieval: Author");
    verify_messages(&msg_inputs_a, retrieved_lists.remove(0))?;
    verify_messages(&msg_inputs_b, retrieved_lists.remove(0))?;
    verify_messages(&msg_inputs_c, retrieved_lists.remove(0))?;
    verify_messages(&msg_inputs_d, retrieved_lists.remove(0))?;

    Ok(())
}

fn split_retrieved(
    retrieved: &mut Vec<UnwrappedMessage>,
    pks: Vec<PublicKey>,
) -> Vec<Vec<UnwrappedMessage>> {
    let mut retrieved_msgs_a = Vec::new();
    let mut retrieved_msgs_b = Vec::new();
    let mut retrieved_msgs_c = Vec::new();
    let mut retrieved_msgs_d = Vec::new();

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
        } else if pk == pks[2] {
            retrieved_msgs_c.push(msg);
        } else {
            retrieved_msgs_d.push(msg);
        }
    }

    vec![
        retrieved_msgs_a,
        retrieved_msgs_b,
        retrieved_msgs_c,
        retrieved_msgs_d,
    ]
}
