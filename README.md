# racehub-rust

# Capture
- Intercepts the udp packets from the game
- Parses the packets into a common format
- Saves the packets to a file
- Saves the packets to a database
- Sends the packets to a grpc server

# Aggregator
spawns grpc server to receive the data from the capture dlls and save to the database and serve the data to the web

## Community use

RaceHub source contributions are available under the scope in [LICENSING.md](LICENSING.md).
Local features do not require a RaceHub subscription.
Third-party games, SDKs, and optional hosting keep their own terms and costs.
See [CONTRIBUTING.md](CONTRIBUTING.md) and [SECURITY.md](SECURITY.md).
No hosted service or maintenance schedule is included.
