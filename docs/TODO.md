# After original implementation is complete.

After the original plan has been completed, there are a few points that have been signaled during development which may need attention after the complete plan has been compelted, which had not been foreseen beforehand. These are:

- **API performance**: When directly interfacing with the API, endpoints such as `/api/auth/login` are blazing fast. When the frontend calls them, latency goes up to about half a second, which makes the process feel clunky and slow. Presumably, this is due to hidden delays of a CORS nature, but it's worth investigating.