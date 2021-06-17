# ckb-time-scripts

Nervos CKB time script including timestamp and block number

### Getting Started

Build contracts:

```sh
capsule build
```

Run tests:

```sh
capsule test
```

### How to Work

The time scripts include two parts: index state type script and info type script.

The index state cell data has two bytes: index(uint8) and `sum_of_time_info_cells`(uint8). Every time the index state cell is updated, the index will increase by one and the index is always guaranteed to be between 0 and `sum_of_time_info_cells`(not include `sum_of_time_info_cells`).

> `sum_of_time_info_cells` is equal to 12, which means there are 12 info cells

The info cell data has two parts: index(uint8) and timestamp(uint32) or block number(u64), so the length of the info cell data will be five or nine. The timestamp or block number corresponding to the index of the index state cell is currently the latest.

For example:

```
0x06604884b8          // the info cell data with index and timestamp

0x050000000000145030  // the info cell data with index and block number
```

> The timestamp and block number are big endian.
