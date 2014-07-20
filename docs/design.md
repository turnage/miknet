miknet Design
===============================================================================

miknet is designed to be as simple as feasible, from the perspective of both
users and developers.

Flow
-------------------------------------------------------------------------------

Programs using miknet interact with the miknetwork through their own miknode.
Every entity in the miknetwork is a miknode. There is no distinction between
servers and clients.

The basic flow of using a miknode is:

* Create.
* Optionally bind.
* Queue data to be sent to peers.
* Routinely service, which dequeues data for peers and receives data from them.
* Handle the data received from peers.
* Close when done.

There are two reasons for not performing network jobs on-demand:

* Many of them are blocking, and it would be confusing to make the
  distinction.
* miknodes would die of inactivity without routine service. Ensuring users
  call it in their program loop is preferrable to forcing them to make their
  program multithreaded.

Dependency Tree
-------------------------------------------------------------------------------

miknet has a simple hierarchy. Modules should have at most 2 dependencies,
not including universal dependencies like logging or nested dependencies.

Below is a sketch of the modules in miknet and their relationship:

````
mikmemapi mikmemmock* mikwebapi mikwebmock*
       |   |                |   |
       |   |                |   |
       |   |                |   |
      \ / \ /              \ / \ /
    mikmemshield        mikwebshield
             |          |
mikpacket    |          |
        |    |          |
        |    |          |
        |    |          |
       \ /  \ /         |
        mikdata         |
              |         |
              |         |
              |         |
             \ /       \ /
                mikpeer
                |
    mikmail     |
         |      |
         |      |
         |      |
        \ /    \ /
         miknode

*modules with an asterisk are testing modules only.
````

The one module not included in this map is ````miklogger````, which all modules
depend on.
