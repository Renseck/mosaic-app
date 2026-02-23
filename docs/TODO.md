# After original implementation is complete.

After the original plan has been completed, there are a few points that have been signaled during development which may need attention after the complete plan has been compelted, which had not been foreseen beforehand. These are:

- **API performance**: When directly interfacing with the API, endpoints such as `/api/auth/login` are blazing fast. When the frontend calls them, latency goes up to about half a second, which makes the process feel clunky and slow. Presumably, this is due to hidden delays of a CORS nature, but it's worth investigating.
- **Edit Layout**: Creating a new panel defaults to coords and dimensions (0,0,1,1), but this leads to overlapping panels when there are already panels there. A full reload is needed to make it look right.
- **Bootstrapping**: Make sure we can inject an API key and put into env or something, or new installs will always be very cumbersome to setup. Having to login to NocoDB, get an API key, put it in env. 