use iota_streams::{
    app::transport::tangle::client::Client,
    app_channels::api::tangle::{Address, Author, Subscriber},
    core::{println, Result},
};

use crate::examples::ALPH9;
use iota_streams::app_channels::api::tangle::PublicKey;
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

    // ------------------------------------------------------------------
    // In their own separate instances generate the subscriber(s) that will be attaching to the channel
    let mut subscriber_a = Subscriber::new("SubscriberA", "utf-8", 1024, client.clone());
    let mut subscriber_b = Subscriber::new("SubscriberB", "utf-8", 1024, client);

    // Generate an Address object from the provided announcement link string from the Author
    let ann_link_split = ann_link_string.split(':').collect::<Vec<&str>>();
    let ann_address = Address::from_str(ann_link_split[0], ann_link_split[1])?;

    // Receive the announcement message to start listening to the channel
    subscriber_a.receive_announcement(&ann_address)?;
    subscriber_b.receive_announcement(&ann_address)?;

    // Send subscription messages linked to announcement message
    let subscribe_msg_a = subscriber_a.send_subscribe(&ann_address)?;
    let subscribe_msg_b = subscriber_b.send_subscribe(&ann_address)?;

    // These are the subscription links that should be provided to the Author to complete subscription
    let sub_msg_a_str = subscribe_msg_a.to_string();
    let sub_msg_b_str = subscribe_msg_b.to_string();

    println!("Subscription msgs:\n\tSubscriber A: {}\n\tTangle Index: {}\n\tSubscriber B: {}\n\tTangle Index: {}\n",
             sub_msg_a_str, subscribe_msg_a, sub_msg_b_str, subscribe_msg_b);

    // These are the public keys that the Author will use to specify specific users in a keyload
    let sub_a_pk = subscriber_a.get_pk().as_bytes();
    let sub_b_pk = subscriber_b.get_pk().as_bytes();
    // ----------------------------------------------------------------------

    // Get Address objects from subscription message links provided by expectant subscribers
    let sub_a_link_split = sub_msg_a_str.split(':').collect::<Vec<&str>>();
    let sub_a_address = Address::from_str(sub_a_link_split[0], sub_a_link_split[1])?;

    let sub_b_link_split = sub_msg_b_str.split(':').collect::<Vec<&str>>();
    let sub_b_address = Address::from_str(sub_b_link_split[0], sub_b_link_split[1])?;

    // Author processes subscribers
    author.receive_subscribe(&sub_a_address)?;
    author.receive_subscribe(&sub_b_address)?;

    // Expectant users are now subscribed and can be included in a Keyload message

    // Author sends Keyload for all subscribers
    let (keyload_all_link, _seq) = author.send_keyload_for_everyone(&announcement_link)?;
    println!(
        "Keyload link for All: {:?}\n\tTangle Index: {}\n",
        keyload_all_link, keyload_all_link
    );

    // Author sends Keyload for just Subscriber A
    let (keyload_a_link, _seq) = author.send_keyload(
        &announcement_link,
        &[],
        &vec![PublicKey::from_bytes(sub_a_pk)?],
    )?;
    println!(
        "Keyload link for Subscriber A: {:?}\n\tTangle Index: {}\n",
        keyload_a_link, keyload_a_link
    );

    // Author sends Keyload for just Subscriber B
    let (keyload_b_link, _seq) = author.send_keyload(
        &announcement_link,
        &[],
        &vec![PublicKey::from_bytes(sub_b_pk)?],
    )?;
    println!(
        "Keyload link for Subscriber B: {:?}\n\tTangle Index: {}\n",
        keyload_b_link, keyload_b_link
    );

    Ok(())
}
