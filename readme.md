miknet
===============================================================================

miknet is a networking library. It is simple to use and requires no networking
knowledge. The api has less than ten functions. No set up, no boilerplate, only
the good stuff.

No distinction is made between clients and servers. All nodes in the miknetwork
begin on equal standing. They can be used as clients and servers, or in more
interesting ways.

Tutorial
-------------------------------------------------------------------------------

The miknetwork is made up of nodes. In order to become a part of it, you need
to be one.

````
miknode_t *node = miknode_create(MIK_IPV4,  /* IPV4 and IPV6 are supported. */
                                 1234,      /* The port to bind to. */
                                 100);      /* Maximum amount of peers. */
````

Congratulations; you're someone. Now, you have two ways of making friends.
You can just wait for other people to greet you, or greet people. To greet
people:

````
int friend = miknode_greet("www.friend.com" /* IP or domain name. */
                           1234);           /* Destination port. */

/* Associate data with friends using their data field. */
node->friends[friend].data = "This is www.friend.com";
````

Now, send your friend a nice letter!

````
const char *letter = "Hello, how are you?";
miknode_mail(friend, letter, strlen(letter));
````

In place of a letter, you can send any kind of data. Provide the address and
length, and miknet will do the rest (and make its own copy of your data, so
it can be discarded after the call).

No serialization of any kind is performed; your data will be delivered
*exactly* as provided, which may or may not be what you intend.

Mail is not sent on-demand. 'mail' is chosen as the function name to avoid
misconceptions about this. The letter is in the mailbox, and the flag is up,
but it hasn't actually sent.

To actually send mail, and receive the mail that others have sent you, you
must service.

````
miknode_service(node,  /* Provide the node. */
                1000); /* Provide a maximum blocking time in ms. */
````

This sends out the mail in your mailbox, and fills yours up with any mail
you've received. You can read your mail in any number of ways, but here is an
example:

````
mikmail_t *mail;
while (mail = miknode_get_mail(node)) {
        if (mail->type == MIK_GREETING) {
                /* New friend! */
                int author = mail->author;
                node.friends[author].data = "New friend!";
        } else if (mail->type == MIK_LETTER) {
                /* New data from friend. */
                do_something(mail->author, mail->content, mail->length);
        } else if (mail->type == MIK_LEAVE) {
                /* Lost a friend. Their data field has been cleared. */
                cry_about_it(mail->author);
        }
};
````

Don't worry about freeing the memory used by mail. miknode_get_mail() will
handle freeing it in the next call.

When you're done, close up shop.

````
miknode_close(node);
````

That's all there is to it! Well, there is more available, but this is all
you *need*. Check the header files for some interesting stuff. If it's marked
as a user space function, you can rely on it remaining in future versions.
There are even some easter eggs!

Installation
-------------------------------------------------------------------------------

To install miknet, you need

* A C89 compiler.
* A POSIX compliant machine.
* cmake

Then, just ````sh install.sh```` or install manually using:

````
mkdir build && cd build
cmake -DCMAKE_BUILD_TYPE=Release ..
make
make install
````

Happy networking!
