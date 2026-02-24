# After original implementation is complete.

After the original plan has been completed, there are a few points that have been signaled during development which may need attention after the complete plan has been compelted, which had not been foreseen beforehand. These are:

- **Grafana panel browser**: Adding a Grafana panel currently requires the user to manually navigate to the Grafana UI, find the panel, copy the embed link, paste it into the panel picker, and trim the URL. Instead, build a panel browser that uses the Grafana API (`GET /api/search?type=dash-db` to list dashboards, `GET /api/dashboards/uid/{uid}` to get panels within a dashboard) and lets the user pick a dashboard → pick a panel from a visual list → auto-constructs the correct `/proxy/grafana/d-solo/{uid}/{slug}?panelId={id}&theme=light` source URL. This would replace the free-text URL input in the panel picker with a structured selection flow.
  
- **Bootstrapping**: Cold starts, as would be done when creating a whole new image and spinning up a stack of containers, come with a few challenges. Currently, an operator needs to create a Grafana service accound and grant it an API token. This value then needs to be gathered and put in the .env file. Also in Grafana, a datasource needs to be configured, and its UID needs to be put in the .env file. Lastly, an API token for NocoDB needs to be created and stored in the .env file as well. There are API endpoints for most or all of these tasks, but they require an API token to work, so we're in a problem of what came first; the chicken or the egg?

- GRAFANA_SERVICE_ACCOUNT_TOKEN=glsa_xxxxxxxxxxxx
- GRAFANA_DATASOURCE_UID=<your-datasource-uid>
- NOCODB_API_TOKEN=xxxxxxxxxxxxx

Tasks that need to be done directly on the Portal can be done:

- Registering an original admin, so the platform can actually be taken into use.

Do you have ideas on how to solve these problems, to make bootstrapping as clean and reproducible as possible?
- **API performance**: When directly interfacing with the API, endpoints such as `/api/auth/login` are blazing fast. When the frontend calls them, latency goes up to about half a second, which makes the process feel clunky and slow. Presumably, this is due to hidden delays of a CORS nature, but it's worth investigating.

- **Automatic portal dashboard generation**: When provisioning a template, a Grafana dashboard is automatically created. We could, with an opt-in question for the user, extend the pipeline to automatically generate a dashboard on the mosaic portal as well, automatically populated with every single panel from the Grafana dashboard, so that the user only needs to drag/resize panels as they see fit - far less manual work.