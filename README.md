<div align="center">

# SquFi

pronounced `sqwa-fy`

![alt text](squfi.jpg)

</div>

SquFi is a crowd-funding protocol built on [solana](https://solana.com/).

Squfi has multiple types of funds a user can create. All funds have a max that when reached depositors will not be able to deposit.

- FundMe
  - FundMe is  exactly what it sounds like. A way to fund yourself. There are countless use cases for this type of fund a few are:
    - Health Bills
    - Tuition
  - FundMe's can also be used to help pool money with your friends for a group trip.

- Raise
  - A raise can be private or public. There are associated shares to a raise. This represents
    - A private raise has a list of address that are allowed to deposit. The owner of the fund has to add the address to the list in order for the depositor to deposit. This can be a way to integrate KYC/AML.
    - A public raise allows anyone to deposit.

- ETF
  - Coming soon..

## Deployment

### Build

For development cargo build can be used:

```sh
cargo build --features program
```

To build the smart-contract for deployment it must be built for the solana BPF target.

> Note: install build-bpf is required. Instructions can be found in the Solana documentation, [here](https://docs.solana.com/cli/install-solana-cli-tools)

BPF Target:

```sh
cargo build-bpf --features program
```

## CLI Walkthrough

TODO

## Features

- [x] Create funding pool
- [x] Add tokens to Funding pool
- [x] Owner of pool withdraw tokens
- [ ] Add pool contributor paybacks
- [x] Create a token to represent pool ownership
- [ ] Ability to create proposals to withdraw money
- [ ] Vote on proposals
- [ ] Rage quit by depositors
