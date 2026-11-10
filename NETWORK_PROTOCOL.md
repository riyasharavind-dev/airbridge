# Protocol draft (not implemented)
Discovery advertises protocol version, ephemeral instance identifier and signaling endpoint via mDNS with bounded UDP fallback. Advertisements are untrusted; do not reveal secrets.

States: idle -> discovered -> pairing pending -> both approved -> authenticated -> negotiating -> streaming -> disconnected. Reject invalid transitions. Timeout and cancel revoke pending authorization.

Authenticated capability negotiation includes roles, codecs, resolution/FPS limits and genuine virtual-output support. Unknown capability is not readiness. Version mismatch fails with explanation. Limit message sizes, rate and lifetime before network implementation. Media transport framing depends on the transport evaluation; do not freeze wire compatibility yet.
