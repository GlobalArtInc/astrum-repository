---
layout: home

hero:
  name: Astrum Repository
  text: One server for every package your builds use
  tagline: Host, proxy and group repositories for 39 formats, from Maven and npm to Docker, Cargo and Helm. One Rust program with PostgreSQL, blob stores on disk or in S3, and a web interface in twelve languages.
  image:
    src: /favicon.svg
    alt: ''
  actions:
    - theme: brand
      text: Quick start
      link: /getting-started
    - theme: alt
      text: Formats
      link: /formats/
    - theme: alt
      text: GitHub
      link: https://github.com/GlobalArtInc/astrum-repository

features:
  - title: 39 formats
    details: Maven, npm, Docker and OCI, PyPI, Cargo, Go, Helm, NuGet, Terraform, APT, Yum, Hugging Face, browser extensions and more, plus remote caches for Nx and Turborepo.
    link: /formats/
    linkText: All formats
  - title: Hosted, proxy and group
    details: Publish your own packages, cache public registries so builds keep working when they are down, and serve both from one address.
    link: /repositories/overview
    linkText: Repositories
  - title: Storage that grows
    details: Blob stores on disk, in S3 or on other servers through an agent, combined into groups and extended without moving data.
    link: /configure/blob-stores
    linkText: Blob stores
  - title: Access under control
    details: Users, roles, privileges and content selectors, LDAP and OpenID Connect sign-in, and access tokens for CI.
    link: /configure/authentication
    linkText: Sign-in and roles
  - title: Built for operations
    details: Health endpoints, Prometheus metrics, scheduled tasks, cleanup policies, audit, backups and several servers on one database.
    link: /operate/monitoring
    linkText: Monitoring
  - title: Agents and automation
    details: A REST API with an interactive reference in the web interface, and an MCP server so agents can search, inspect and administer.
    link: /use/mcp
    linkText: Agents over MCP
---

<div class="home-shot">

![The Astrum Repository dashboard](./images/dashboard-light.png){.light-only}
![The Astrum Repository dashboard](./images/dashboard-dark.png){.dark-only}

</div>
