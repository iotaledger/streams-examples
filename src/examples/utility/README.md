# Utility Examples
These examples highlight specific pieces of Streams functionality. 

### [Fetch Previous Messages](fetch_prev.rs)
An example of how to fetch previous messages (provided the user has access to these messages).

### [Grant and Revoke Branch Access](grant_and_revoke_access.rs)
An example of how to grant and revoke access to a branch. This can only be done in multi branch channels 
as the Keyload messages will need to be attached to sequencing messages if they are intended to be read 
by a user that did not originally have access to a branch. Retroactively granting access is not supported.  

### [Issuing Keyloads Using Public Keys](pk_keyloads.rs)
The traditional/suggested way of issuing read/write access to a branch using subscriber public keys. 

### [Issuing Keyloads Using Pre Shared Keys](psk_keyloads.rs)
A quick example of how to create a Pre Shared Key and use it in a Keyload message to grant read access
to a particular branch.

### [Exporting and Importing a User State](state_recovery.rs)
Exporting a user state regularly is good practice, it acts as a snapshot of the current message states 
of the known publishers in a channel. Stored states can be used to quickly reboot a channel instance or 
pass it between application instances. Exported states are password protected. 

### [Recovering a User Without a State](stateless_recovery.rs) 
For implementations where a state cannot (or should preferably not) be stored, account recovery can be
done provided the user has:
- The seed of the user they are recovering
- The announcement address of the channel 
- The Channel Type [Author only]
- A client
