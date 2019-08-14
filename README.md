# mpc

This is a multi-party computation protocol for the key-generation step of Pinocchio zkSNARKs [[PGHR13]](https://eprint.iacr.org/2013/279) designed for use in the Zcash "Sprout" public parameter ceremony.

## Zcash Ceremony

The public parameters (zk-SNARK proving and verifying keys) for Zcash's 1.0 "Sprout" launch
were constructed in a ceremony that took place on October 22-23.

The following individuals participated in the ceremony:

* Andrew Miller
* Peter Van Valkenberg
* John Dobbertin
* Zooko Wilcox
* Derek Hinch
* Peter Todd

The ceremony used a multi-party computation protocol with the property that the resulting
parameters are secure unless _all_ of the participants were dishonest or compromised
during the ceremony.

### Assets

[`r1cs`](https://download.z.cash/zcashfinalmpc/r1cs): 6111c2bce234867201d887170d68712c8f8785a1c97d43ab4ca540d7704ee8f7

[`transcript`](https://download.z.cash/zcashfinalmpc/transcript): 7da0c07a4bec04fbe4ae99ebd62d4ce7e1710b1f7a1f317345b0a48feec984d3

[`sprout-proving.key`](https://download.z.cash/zcashfinalmpc/sprout-proving.key): 8bc20a7f013b2b58970cddd2e7ea028975c88ae7ceb9259a5344a16bc2c0eef7

[`sprout-verifying.key`](https://download.z.cash/zcashfinalmpc/sprout-verifying.key): 4bd498dae0aacfd8e98dc306338d017d9c08dd0918ead18172bd0aec2fc5df82

[`finalmpc2-compute.iso`](https://download.z.cash/zcashfinalmpc/finalmpc2-compute.iso): 5f43aa1244a01b3cf9da4abeadde9e34b954a873565fc56b58c10780f3ce0e4c

[`finalmpc2-network.iso`](https://download.z.cash/zcashfinalmpc/finalmpc2-network.iso): 375550be4c64ebc68a9306421bb71ad3556bc73f156a231503084f923900f4cb

#### r1cs

This is the rank-1 quadratic constraint system used by Zcash. You can use Zcash to construct it with the following PR: <https://github.com/zcash/zcash/pull/3691>.

#### transcript

This is a transcript of the protocol that can be used to verify the protocol's evaluation and construct the proving/verifying keys.

#### sprout-*.key

These are the final parameters that can be built from the transcript.

#### finalmpc2-*.iso

These are reproducibly generated Live CD images used during the ceremony, using a modified [Alpine Linux](https://www.alpinelinux.org/) distribution.

The minimal operating system is patched with [grsecurity](https://grsecurity.net/), an extensive security enhancement to the Linux kernel. The compute nodes have a restrictive RBAC (role-based access control) policy which is intended to allow *only* the execution of code and granting of permissions that are necessary for its function. The ISOs are built inside [Docker](https://www.docker.com/) containers.

### Code and verification

The code used for the ceremony was tagged at `finalmpc2`. It's mostly written in [Rust](https://www.rust-lang.org/).

#### ISOs

The participants all booted from the `finalmpc2-compute.iso` on hardware they obtained securely. It can be reproducibly built using the `build-iso.sh` tool. Some of the participants also used `finalmpc2-network.iso` for the networking during the ceremony, though it was not necessary.

#### Transcript verification

Given `r1cs` and `transcript`, and powerful enough hardware, you can verify the protocol transcript and construct `pk`/`vk` within a few hours. Run `cargo run --release --bin verifier`.

Here is the log from verifying the transcript:

```
Number of players: 6
Player 1 commitment: 2iQQBkf7k4K9aigJm4uHHufaSB8rXLLaRTMmTerK7dx6RCqNc9
Player 2 commitment: 6yV3Ji7zuVWVCQEfkhQ6Vfv51t5VfQHQVaLDGH6zkeunKmohr
Player 3 commitment: 6mGvvMFMKJNwKFmHXUwcCQMk7iu92bSqhtRabX3nkdnadEKte
Player 4 commitment: VGyYjzYc39em9TithdWFySSUwATMgcXcLtQ7ias7i4SkNdS4G
Player 5 commitment: 2YrFsjMadFukhdkQpn8oFgET2EQd9WnDW3AzYqNc3kELU45p7t
Player 6 commitment: 2B2HXuZAKayqgJpxojuUU9RN78pTv2gLvEDmEbWRBWEJ6Z1LS
Player 1 hash of disk A: 2oX6hBNiQxiZYZgDbSkgk3mhBACXmoGCfdhfZrSquNztZuZaqt
Player 1 hash of disk B: 2T2ceUDomnrCVCtJw2SwtYAeHCfnAhM9HBdzVkq6BdZ59nST5m
Player 2 hash of disk A: 2axdkGL6QzngjvY9jRBX5AqhSokukji8eQuYUfJwhp7sxcXvPr
Player 2 hash of disk B: 2RymyNbAWaBVDuzW4m1iKA72MsmZFnwMhNvqxxXDwugLTa62wc
Player 3 hash of disk A: YAQs9PiruKxfTwMTdTUHkYgt9QRvjpkF95cJRbNP8WLqPqjLW
Player 3 hash of disk B: 2omA7bsepmmxeygQzNBbodhdXTyhuK1i2KCRH9esB3azunwZPn
Player 4 hash of disk A: EDEtkk1PUhu4BQbTzx5yPSSpyqB6kV9g39p6sNt1ERGRG3APQ
Player 4 hash of disk B: 2fvnbP22XWHVD1DstGQ5FsHNaBLiZQg4MBVKmWf7sWCYg5A9L7
Player 5 hash of disk A: 2oQgZxPLAL2f8xkvm71RqwKK6dCFQSrazESXci32M2LZeG7nxe
Player 5 hash of disk B: UBjr6UU8oJ4ZzpsTU3vRHmzZmuN7TjX3eLsmdRhw4dW6dEbvH
Player 6 hash of disk A: rnMAJE2bxMbCT6yRvufD2ww17kmP9qaKnipxrvZTWXe27d6GW
Player 6 hash of disk B: 27Em5cp6QSGVsJsAcvZLW7CoMkKv5Ybi3LAGPPeGwqCF7Diex
Player 1 hash of disk C: 29PLu7dtT9BjJhtoAzxpSxvrp4tE15xjJufL1ANHGwkwieyxMo
Player 1 hash of disk D: 2YVAELKtHdufKRPTzT5ZpHFgxrcro6JmBKkYz4GEQqcXbQdViM
Player 2 hash of disk C: 2qSQhJvQLjmXfQWHKMCR5EukSWU9BQ3KwdPSqkPUCSRzwmxowM
Player 2 hash of disk D: 2uAxySzeptYhEowKuBRGituPnc1U4BU1GMuL4Hfbyvtgq7x4Qn
Player 3 hash of disk C: 2DZ3pnkZcTMfAa28KfJpD5fQbnkQZZG5mFnFqvHHUDXJquSJAX
Player 3 hash of disk D: 2tfcauKUDBirJFSo8jbyEenLfHULThsQjVdN8FY3hJGn3dC2JP
Player 4 hash of disk C: 2gH6XJ8BeA5yXZL95ThSp9ucicwAoevaDK6xNBck9QUxXF1gEE
Player 4 hash of disk D: VFXKdDoYrA58evyJUrvkocGCHVvYF2h8HVLmuEtFkDfZY6EHk
Player 5 hash of disk C: 2RvKUp94tXE5b1qhyLpGPTXeWpS7FdNDvCG5MJPmZiccNuRYcw
Player 5 hash of disk D: ApPFWMqGBMemE3sTAuMRnwbmGonsPoXYC4r45HBMdmiRWLXqH
Player 6 hash of disk C: 2wGGYBXaeQdbLHvViArnLkGRERhztVk5qZmreSKwxEcFjMBNMC
Player 6 hash of disk D: 2g3rMWwyCL5wgKYHiVHR6EdnBc9Q5dPc1RW6tWwvwyJnx6AKq9
Player 1 hash of disk E: 2Zbrd1XhYKvZqeNcGQPVrusx1rRjxaQjFfzWcn64wCfTnEGTMg
Player 1 hash of disk F: 27ZzVxLTxXpjeTo86sdQ9kKU83UfNHLyGPuQ4CCV9ZRJ4g84jC
Player 2 hash of disk E: YWKCeTeYiKUNnd4aJBYcd8ZwBxscibmtDa4pxbz52fpYX2H9S
Player 2 hash of disk F: 2o1wWJHYzCirDmijHmnGFQ4pSfoYTkEKdPinag22eYonKf8EGC
Player 3 hash of disk E: 2jquuLB8omrtWV1GnXvghRN1A3MWMouyBSwEKD5fCMwk5SvktP
Player 3 hash of disk F: 2jrEGwnSyX9oX8UUGYhpEiPaLmGmrbhfFtcciXt3o5N7nPh63A
Player 4 hash of disk E: AY1Vm8dDSxDdpNhac8Mr7GkS18vomvXaoreg1mVcXyApmgbu8
Player 4 hash of disk F: 2RVi4vpjXtzD6gPLsFDSVrtX545HbVnNBhjAJVUTXpG22oLDD5
Player 5 hash of disk E: CFEWpN9STr4iVM8NLGcSUyoaEDr94FEp7VWR9HhQQYhuwUu7f
Player 5 hash of disk F: 2vohW4tyybTEZyf3ZarX5R1CgsUehQfwASExZQ86EWNd8ByC6a
Player 6 hash of disk E: chZdF1yRVDTsaD14KdaFv6N7e8ZPkMnxr9CpXkzq8JzonhLPx
Player 6 hash of disk F: 2HjRqGyKjPxDSbhP8KgyYtKpWCwrGt3v4ZEUZHsZpJHbJ2V9QL
```

## License

Licensed under either of

 * MIT license, ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.

This dual license applies to all of the files listed in the assets section of this `README`.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.

## Authors

* [Sean Bowe](https://github.com/ebfull)
* [Ariel Gabizon](https://github.com/arielgabizon)
* [Matthew Green](https://isi.jhu.edu/~mgreen/)
