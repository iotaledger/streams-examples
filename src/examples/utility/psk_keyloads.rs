use iota_streams::{
    app::transport::tangle::client::Client,
    app_channels::api::tangle::{Address, Author, ChannelType, Subscriber},
    core::{println, psk::Psk, Result},
};

use crate::examples::ALPH9;
use rand::Rng;

/**
 * This example demonstrates how to use a Pre-Shared Key for accessing a branch. PSK's should only
 * be used for read access at this time.
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

    // Generate a key to be used as a Pre Shared Key
    let key = rand::thread_rng().gen::<[u8; 32]>();

    // Author will now store a PSK to be used by Subscriber B. This will return a PskId (first half
    // of key for usage in keyload generation)
    let psk = Psk::clone_from_slice(&key);
    let pskid = author.store_psk(psk);

    // ------------------------------------------------------------------
    // In their own separate instances generate the subscriber(s) that will be attaching to the channel
    let mut subscriber = Subscriber::new("SubscriberA", client);

    // Generate an Address object from the provided announcement link string from the Author
    let ann_link_split = ann_link_string.split(':').collect::<Vec<&str>>();
    let ann_address = Address::from_str(ann_link_split[0], ann_link_split[1])?;

    // Receive the announcement message to start listening to the channel
    subscriber.receive_announcement(&ann_address)?;

    // Store the PSK in the Subscriber instance
    let _sub_pskid = subscriber.store_psk(Psk::clone_from_slice(&key));
    // ----------------------------------------------------------------------

    // Author sends Keyload with PSK included
    let (keyload_all_link, _seq) = author.send_keyload(&announcement_link, &[pskid], &vec![])?;
    println!(
        "Keyload link for All: {:?}\n\tTangle Index: {}\n",
        keyload_all_link, keyload_all_link
    );

    Ok(())
}
