# Multi Publisher Examples 
These examples are intended to give an overview on different approaches for having multiple publishers 
in a channel. It is highly recommended that one does not have multiple concurrent publishers within a 
`Single Branch` channel. When there are multiple publishers within the same branch of a `Multi Branch` 
channel, it is important to make sure that each publisher is synchronising their state before publishing, 
otherwise there could be errors in sequencing, and the subscribers may fail to find/publish messages. 

### [Single Publisher Per Branch](single_pub_per_branch.rs)
Author generates a channel where each subscriber added is given its own branch to publish in. This is 
done by sending a new `Keyload` for each new Subscriber in the channel that they can then link their 
messages to.

### [Multiple Publishers Per Branch](multi_pub_per_branch.rs)
Author generates a channel where two subscribers are added to each of two branches. Subscribers A and B 
post their messages in alternating order in branch A, demonstrating the synchronisation between each 
publishing entity to keep states in check. The same is done for Subscribers C and D in branch B. 
