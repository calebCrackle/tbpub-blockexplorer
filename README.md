# TBPUB

## Bitcoin Core

### Description

Bitcoin core is the only fully trusted p2p network, such that when a transaction is created containing data its geuarenteed to proporgate across the network. The data type bitcoin core uses is a blockchain, which is a essentally a stack of blocks where the blocks contain a list of transactions. A new block is created and broadcast every ten minutes or so and can contain a couple thousand transactions.

```mermaid
graph LR;
    subgraph BOne [Block 1]
        direction TB
        TOne[Transaction]
        TTwo[Transaction]
        TThree[Transaction]
        TOne ~~~ TTwo ~~~ TThree;
    end
    subgraph BTwo [Block 2]
        direction TB
        TFour[Transaction]
        TFive[Transaction]
        TSix[Transaction]
        TFour ~~~ TFive ~~~ TSix;
    end
    subgraph BThree [Block 3]
        direction TB
        TSeven[Transaction]
        TEight[Transaction]
        TNine[Transaction]
        TSeven ~~~ TEight~~~ TNine;
    end
    BOne --> BTwo --> BThree;
```

### Transactions

Transactions are the only method of communication accross Bitcoin Core. All transactions have at least one input and one output.

```
{
    Inputs: [
        {
            txid: TXID, 
            vout: VOUT
        },
        .
        .
        .
    ],
    Outputs:[
        {
            "script/address": amount
        },
        .
        .
        .
    ]
}
```

### OP_RETURN
Any Output Script that starts with OP_RETURN(0x6a) is unspendable and concidered destroyed. This isnt harmful to Bitcoin because by destroying some Bitcoin, you make all the rest of it more rare and therefor more valuable. You can think of it as a donation to everyone that has bitcoin. 

### Limitations

Bitcoin Core requires a node to be online almost all the time to recieve new blocks.
Every transaction is transmited and stored on every node therefor we cannot store much data in the network.
Data sent through a Bitcoin Transaction is generally hundreds of times more expensive then data sent through tbPUB.


## tbPUB Transaction

tbPUB uses a certian type of Bitcoin Transaction that is used to discover Root Nodes and Published Books. 
All tbPUB Transactions have one output that sends to a script constructed as follows:
1. (RED)    OP_RETURN(0x6a)
2. (GREEN)  The text "TBPUB" hex encoded
3. (BLUE)   A 0 to indicate a Root Node URI OR a 1 to indicate a Book Hash
4. (PURPLE) The URI or Book Hash as indicated above

### Examples

A Root Node listing pointing to http://example.com
![resources](https://docs.google.com/drawings/d/e/2PACX-1vSgik6ZBIjJLcii07vS8Lk9De2VsoUDsqZgCeFFlZUg-Y6JljoVq3MVhRcCifDtEe6UvsxYu4Fkl32a/pub?w=480&h=360)

A Book Hash listing with the hash:
323072616E646F6D323063686172616374657273

![resources](https://docs.google.com/drawings/d/e/2PACX-1vT0izA1a35zSeAtRyYYDBRgBd2YQsrxlRDzxnFwIh0nOmjLDRHmWI3egeqaeVCIgaHQNzKt1K1EXrZz/pub?w=480&h=400)

The amount of BTC that was sent to the script starting with OP_RETURN is the Cost of the Transaction. If the transaction is publishing a Root Node URI that cost must be at least 10,000 sats to be valid. If the transaction is publishing a Book Hash the cost must be at least 1 sat per byte in the book. Any transactions that don't meet this requirement are ignored as invalid.

### Limitations
As its creating an unspendable Transaction Output we have to be very limited in the amount of data we store and how many of these tbPUB Transaction we create. Therefore we limit the protocal to one tbPUB Transaction per Block. If two or more tbPUB Transacactions are found only the highest paying one is concidered valid and the rest are ignored. This provides a large incentive to batch up documents before publishing and ensure that there are no other tbPUb Transactions in he mempool before broadcasting.

## tbPUB Root Node 

The tbPUB Root Node is the equivalent of a Bitcoin Core full node, it is split into two major components, the block explorer, and the hosted DWN.

### Block Explorer
The block explorer requires a connection to a Bitcoin Core RPC instance and serves to purposes. The first is it will crawl through the blockchain looking for any other Root Node DIDs, and Book Hashes. Storing these DIDs and Hashes in a SQLITE database at a configurable location. The second is to use a wallet to create new transactions for broadcasting DIDs and Book Hashes on the Block-chain.

#### Config Options

##### bitcoinrpcuser(Required)
This is the user name for the running Bitcoin Core RPC instance

##### bitcoinrpcpassword(Required)
This is the password for the running Bitcoin Core RPC instance

##### wallet(Required)
This is the wallet name to use for transaction creation

##### datadir(Optional)
This is a path to a directory to store the databases and read the config file. Defaults to ```/home/user/.tbpub/```

##### cliurl(Optional)
This is the url to listen for RPC requests for the block explorer. Defaults to ```localhost:9443```

##### bitcoinrpcuser(Optional)
This is the url for the running Bitcoin Core RPC instance

### Joining the Network
After we get a list of Book Hashes we need to resolve the Hash to actual Books. To prevent DDOS attacks Root Nodes can only be queried by other Nodes on the Network. To join the network the Root Node will create a tbPUB Transaction containing the URI to your Root Node.

Spam/DDOS Problems:
To prevent a Root Node from being bombarded with querys for Books each Root Node will only share books with another Root Node, And will only pass it along to each Root Node on the Network once.

### Resolving Book Hashes
To get a book the Root Node will query other Root Nodes for the Book coropsonding with the Hash. After reciveving a Book it will verify the hash matches before marking it as a valid Book. 

Spam/DDOS Problems:
2. To prevent a Root Node from reading a ton of garbage data when querying for a book, Root Nodes will be concius of the amount paid for the Book in the tbPUB Transaction ensuring it never attepmts to read more data then was paid for.

### Datatypes

#### Page

An object that contains only a price and a data field. The price of the Page is equivilent to 1 Satoshi per Byte in the data field.

```JSON
{
    "price": 10,
    "data": "1234567890"
}
```

#### Book

A JSON Array of Pages. The price of the book is the sum of the prices of its pages.

```JSON
[
    {
        "price": 10,
        "data": "1234567890"
    },
    {
        "price": 20,
        "data": "12345678901112131415"
    },
    .
    .
    .
]

```
