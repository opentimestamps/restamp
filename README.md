# The Restamp Project

While the OpenTimestamps client rounds off Bitcoin block times to the nearest
day, in some cases more precision is desirable. It's also desirable to have
additional sources of attestations on top of Bitcoin block headers, in the
unlikely case that Bitcoin miners choose to use inaccurate timestamps in their
blocks.

To that end, the Restamp project aims to timestamp Bitcoin blocks in additional
ways. At the moment, that consists of using the Roughtime protocol to timestamp
Bitcoin blocks as they are mined, using Bitcoin Core's `-blocknotify` feature
on a fast, well-connected, Bitcoin node. These timestamps are archived in the
`roughtime` directory of this repo; the code to stamp and verify these
timestamps is in `roughstamp`. The git commits of this repo are additionally
timestamped, ensuring the trusted Roughtime signatures can be validated in the
future even if the keys are leaked.

Note that this project works in conjunction with [nist-inject](https://github.com/opentimestamps/nist-inject).
Together, they constrain both how old and how new a Bitcoin block could be.
