# Abstract store object

Nix organizes the data it manages into *store objects*.
An abstract store object in the abstract is a black box that can reference other store object.

In pseudo-code:

```idris
data StoreObject
data StoreObjectRef

getReferences : StoreObject -> Set StoreObjectRef
```

References must not "dangle": in whatever context the object with the reference exists, the object being pointed to must also exists.
This invariant implies that store objects and their references form a graph.
Furthermore, this invariant combined with the immutability of store objects implies that the graph is acyclic:
to avoid both dangling references and mutation, any referenced objects must exist prior to the creation of objects.
By induction then, store objects with references must be "built up** from store objects without them, and this makes the graph acyclic.

Other operations impacted by these invariant are:

**Copying**
Store objects can be copied between stores.
Store objects being copied must refer to objects already in the destination store.
Recursive copying must either proceed in dependency order or be atomic.

**Deleting**
We can only safely delete unreferenced objects.
Recursive deleting (garbage collection) also must either proceed in dependency order or be atomic.
