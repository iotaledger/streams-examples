# Single Publisher Examples 
These examples are intended to give an overview on different approaches for having a single publisher 
in a channel. 

### [Public Single Branch](single_branch_public.rs)
The most basic usage of Streams. Author generates a public channel that anyone can read from once they
have the `Announce` message link.

### [Private Single Branch](single_branch_private.rs)
A private branch with predefined user access. Subscribers are required to properly subscribe to be granted
access to the messages published by the `Author`.

### [Mixed Access Multi Branch](multi_branch_mixed_privacy.rs)
A more complex implementation showing a mix of private and public access in a `Multi Branch` channel. Three 
branches are generated with different message chains and access restrictions. Subscribers are defined:
- Subscriber A: Traditionally subscribed and granted access to branch A (can also read public branch C)
- Subscriber B: Able to read from branch B through use of a `Pre Shared Key` (can also read public branch C)
- Subscriber C: Not properly subscribed at all and only able to read from public branch C

