# tbPUB

## Terms

### Page

A Page is a JSON Object that contains a price and data field. The minimum price for a page is 1 Satoshi Per Byte of data in the data field.
```
    {
        "price": 10,
        "data": "123456789"
    }
```

### Book
A Book is a JSON Array of Pages. The price of a Book is the total Price of all its Pages summed.

### tbPUB Root Node
A tbPUB Root Node is the core of the tbPUB Network, Used for publishing, reading, and verifying Books.

### DID-DOC

A DID-Document is a Web5 document used to point to a service.

### DID-DHT key

A DID-DHT key is a key that resolves to a DID-DOC via a Web5 Gateway node.

### tbPUB Transaction
A tbPUB Transaction is a Bitcoin Transaction used to publish data with a couple of rules:
1. A tbPUB Transaction must destroy at least 10,000 Sats
2. There can only be one tbPUB Transaction per Block. If two transactions are found in the same block, the highest paying one is considered valid while the rest are not.

### DID Transaction

A DID Transaction is a tbPUB Transaction that publishes a DID-DHT key pointing to a Root Node.

### Hash
A Hash is a 20-character string and is unique for every piece of data. The Hash is used to verify a Book hasn't been tampered with.

### Hash Transaction
A Hash Transaction is a tbPUB Transaction that publishes a Book Hash. 

## tbPUB Root Node

### Abstract
tbPUB Root Nodes are how Books get published, read, and verified.

### Description
A tbPUB Root Node is a fully trustless Node that requires Bitcoin Core, the four major things it does are:
1. Join the tbPUB Network
2. Discover other Root Nodes
3. Discover and verify Books
4. Host Books for other Root Nodes

### Joining the tbPUB Network
To join the network, your Root Node must be discoverable. To become discoverable, it will create a DID Transaction with its own DID-DHT key.

### Discovery of Root Nodes
1. Look through the blockchain for DID Transactions
2. Resolve the DID-DHT key with a Web5 Gateway Node

### Discovery of Books
1. Look through the blockchain for Hash Transactions
2. Resolve the Book Hash with another Root Node
3. Ensure the Book hasn't been tampered with
4. Ensure the number of Satoshis Destroyed by the Hash Transaction covers the cost of the Book

### Uses
1. Trustlessly publishing any data
2. Trustlessly verifying published data

### Hosting Books
After Publishing or Discovering a new Book, the Root Node will store and host it.

## TBPUB Miner

### Abstract
A tbPUB Miner is a service that runs on top of a Root Node, allowing for more options with the network.

### Description
A tbPUB Miner uses an API to accept requests to publish or read data from the Root Node.

### Uses
2. As a service that can charge to publish or read data.
3. Used by app providers so their app can communicate with the tbPUB network.

## TBPUB Node

### Abstract 
A tbPUB Node is a lightweight version of the Root Node that can only read and verify Books.

### Description
It works exactly like a Root Node, except it's incapable of publishing data.
Instead of requiring Bitcoin Core, the node accepts blockchain data from a service like blockstream.info.
It reads the Books from a tbPUB Miner, using the blockchain data to verify the books haven't been tampered with.

### Uses
1. By Apps that trust or run a tbPUB Miner 

## TBBandwidth

### Abstract
TBBandwidth is a protocol built on top of tbPUB to allow for the discovery of DIDs and the assertion of a cost.

### Description
A TBBandwidth is a type of tbPUB Page that contains a bunch of DIDs and a Price Per did.
```
{
    "price": 20000,
    "data": [
        {"did-dht:examplekey1": 10000}
        {"did-dht:examplekey2": 10000}
    ]
}
```

### Uses
1. Discovery of DIDs
2. Putting a price on a DID
