# Streams Examples
Here you can find a list of examples to help you with your [Iota Streams](https://github.com/iotaledger/streams) 
integrations. The examples have been broken down into 3 sections:
 
- [Single Publisher](#single-publisher-examples) 
- [Multi Publisher](#multi-publisher-examples)
- [Utility](#utility-examples)

All of the examples are ready to be run in the `src/main.rs` file and can be tested by simply running: 

```
cargo run --release
```

These examples default to sending to `https://chrysalis-nodes.iota.org` which is a load balancer. If you 
would like faster performance it is recommended that you change this node url to your local/private node 
in the `src/main.rs` file before running. 

## Single Publisher Examples 
#### [link](src/examples/single_publisher)

These examples are intended to give an overview on different approaches for having a single publisher 
in a channel. 

#### [Public Single Branch](src/examples/single_publisher/single_branch_public.rs)
The most basic usage of Streams. Author generates a public channel that anyone can read from once they
have the `Announce` message link.

#### [Private Single Branch](src/examples/single_publisher/single_branch_private.rs)
A private branch with predefined user access. Subscribers are required to properly subscribe to be granted
access to the messages published by the `Author`.

#### [Public Single Depth](src/examples/single_publisher/single_depth_public.rs)
A public index retrievable channel. Author generates a public channel that anyone can read from once they
have the `Announce` message link. Subscribers can retrieve messages using an anchor message link and message 
number.

#### [Private Single Depth](src/examples/single_publisher/single_depth_private.rs)
A private index retrievable channel with predefined user access. Subscribers are required to properly subscribe 
to be granted access to the messages published by the `Author`. Once approved, Subscribers can retrieve messages 
using an anchor message link and message number. 

#### [Mixed Access Multi Branch](src/examples/single_publisher/multi_branch_mixed_privacy.rs)
A more complex implementation showing a mix of private and public access in a `Multi Branch` channel. Three 
branches are generated with different message chains and access restrictions. Subscribers are defined:
- Subscriber A: Traditionally subscribed and granted access to branch A (can also read public branch C)
- Subscriber B: Able to read from branch B through use of a `Pre Shared Key` (can also read public branch C)
- Subscriber C: Not properly subscribed at all and only able to read from public branch C


## Multi Publisher Examples 
#### [link](src/examples/multi_publisher)
These examples are intended to give an overview on different approaches for having multiple publishers 
in a channel. It is highly recommended that one does not have multiple concurrent publishers within a 
`Single Branch` channel. When there are multiple publishers within the same branch of a `Multi Branch` 
channel, it is important to make sure that each publisher is synchronising their state before publishing, 
otherwise there could be errors in sequencing, and the subscribers may fail to find/publish messages. 

#### [Single Publisher Per Branch](src/examples/multi_publisher/single_pub_per_branch.rs)
Author generates a channel where each subscriber added is given its own branch to publish in. This is 
done by sending a new `Keyload` for each new Subscriber in the channel that they can then link their 
messages to.

#### [Multiple Publishers Per Branch](src/examples/multi_publisher/multi_pub_per_branch.rs)
Author generates a channel where two subscribers are added to each of two branches. Subscribers A and B 
post their messages in alternating order in branch A, demonstrating the synchronisation between each 
publishing entity to keep states in check. The same is done for Subscribers C and D in branch B. 


## Utility Examples 
#### [link](src/examples/utility)
These examples highlight specific pieces of Streams functionality. 

#### [Fetch Previous Messages](src/examples/utility/fetch_prev.rs)
An example of how to fetch previous messages (provided the user has access to these messages).

#### [Grant and Revoke Branch Access](src/examples/utility/grant_and_revoke_access.rs)
An example of how to grant and revoke access to a branch. This can only be done in multi branch channels 
as the Keyload messages will need to be attached to sequencing messages if they are intended to be read 
by a user that did not originally have access to a branch. Retroactively granting access is not supported.  

#### [Issuing Keyloads Using Public Keys](src/examples/utility/pk_keyloads.rs)
The traditional/suggested way of issuing read/write access to a branch using subscriber public keys. 

#### [Issuing Keyloads Using Pre Shared Keys](src/examples/utility/psk_keyloads.rs)
A quick example of how to create a Pre Shared Key and use it in a Keyload message to grant read access
to a particular branch.

#### [Exporting and Importing a User State](src/examples/utility/state_recovery.rs)
Exporting a user state regularly is good practice, it acts as a snapshot of the current message states 
of the known publishers in a channel. Stored states can be used to quickly reboot a channel instance or 
pass it between application instances. Exported states are password protected. 

#### [Recovering a User Without a State](src/examples/utility/stateless_recovery.rs) 
For implementations where a state cannot (or should preferably not) be stored, account recovery can be
done provided the user has:
- The seed of the user they are recovering
- The announcement address of the channel 
- The Channel Type [Author only]
- A client
