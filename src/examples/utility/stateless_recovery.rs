use iota_streams::{
    app::transport::tangle::client::Client,
    app_channels::api::tangle::{Author, Bytes},
    core::{println, Result},
};

use crate::examples::ALPH9;
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
    let mut author = Author::new(seed, "utf-8", 1024, false, client.clone());

    // Create the channel with an announcement message. Make sure to save the resulting link somewhere,
    let announcement_link = author.send_announce()?;
    println!(
        "Announcement Link: {:?}\nTangle Index: {}\n",
        announcement_link, announcement_link
    );

    // Author will now send signed encrypted messages in a chain
    let msg_inputs = vec!["Send", "Some", "Messages"];

    let mut prev_msg_link = announcement_link.clone();
    for input in &msg_inputs {
        let (msg_link, _seq_link) = author.send_signed_packet(
            &prev_msg_link,
            &Bytes::default(),
            &Bytes(input.as_bytes().to_vec()),
        )?;
        println!("Sent msg: {}", msg_link);
        prev_msg_link = msg_link;
    }

    // Recover from scratch
    let mut new_author =
        Author::recover(seed, &announcement_link, false, client)?;

    let (_last_msg_link, _seq) = new_author.send_signed_packet(
        &prev_msg_link,
        &Bytes::default(),
        &Bytes("One last message".as_bytes().to_vec()),
    )?;

    let msgs = author.fetch_next_msgs();
    if msgs.is_empty() {
        panic!("Old author could not fetch next msg")
    }

    Ok(())
}
