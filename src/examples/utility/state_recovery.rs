use iota_streams::{
    app::transport::tangle::client::Client,
    app_channels::api::tangle::{Author, Bytes, ChannelType},
    core::{println, Result},
};

use crate::examples::{verify_messages, ALPH9};
use rand::Rng;

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
    println!(
        "Announcement Link: {}\nTangle Index: {:#}\n",
        announcement_link, announcement_link
    );

    // Author will now send signed encrypted messages in a chain
    let msg_inputs = vec!["Send", "Some", "Messages"];

    let mut prev_msg_link = announcement_link;
    for input in &msg_inputs {
        let (msg_link, _seq_link) = author.send_signed_packet(
            &prev_msg_link,
            &Bytes::default(),
            &Bytes(input.as_bytes().to_vec()),
        ).await?;
        println!("Sent msg: {}, tangle index: {:#}", msg_link, msg_link.to_msg_index());
        prev_msg_link = msg_link;
    }

    // Export State of author
    let state = author.export("Password").await?;
    // Write state to file
    std::fs::write("./author_state.bin", state)?;

    // Retrieve State from file
    let state = std::fs::read("./author_state.bin")?;

    // Import state
    let mut new_author = Author::import(&state, "Password", client).await?;

    let (last_msg_link, _seq) = new_author.send_signed_packet(
        &prev_msg_link,
        &Bytes::default(),
        &Bytes("One last message".as_bytes().to_vec()),
    ).await?;

    let retrieved = new_author.fetch_prev_msgs(&last_msg_link, msg_inputs.len()).await?;
    verify_messages(&msg_inputs, retrieved)?;

    Ok(())
}
