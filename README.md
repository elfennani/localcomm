# Local Communication
An experimental service to control and communicate with 
other devices locally in the same network in a cross-platform
manner. This is intended as an attempt to recreate the functionality
of KDE Connect. This works by exposing a Bonjour/mDNS service for
discovery and a server for IPC. The project includes a CLI to make use of
the service.

Currently, it lacks any security measures like TLS or a pairing flow.