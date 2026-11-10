# Security design
Threats: forged discovery, PIN guessing, unauthorized capture, replay, malicious packet sizes and dependency substitution. Treat LAN as hostile.

Use audited authenticated key exchange and encrypted media; a short PIN must use a vetted PAKE or authenticated verification workflow, never serve as an encryption key directly. Persist trusted identities in OS credential storage. Require both approvals, rate-limit pairing, expire attempts, bind approvals to peer identities and session nonce. Validate lengths/version before allocation. Reject replay and unauthorized control messages.

Bind only intended LAN interfaces, no port forwarding or cloud relay by default. Stop capture on disconnect. Do not log PINs, keys or screen contents. Driver installs and display changes require explicit consent. No dependency downloader exists in Phase 1.
