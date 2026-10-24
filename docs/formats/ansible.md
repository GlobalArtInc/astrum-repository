# Ansible Galaxy

Repositories serve Ansible collections through the Galaxy v3 API that `ansible-galaxy` speaks: hosted
repositories take `ansible-galaxy collection publish`, proxies cache another Galaxy server such as
`https://galaxy.ansible.com`, and groups merge the versions of their members. Name the repository in
`ansible.cfg` next to your playbooks or in `~/.ansible.cfg`; the token is an
[access token](../configure/authentication.md#access-tokens) of this server (`username` and `password`
work too) and can be left out where anonymous users may read the repository:

```ini
[galaxy]
server_list = astrum

[galaxy_server.astrum]
url = https://astrum.example.com/repository/ansible-group/
token = <access token>
```

```shell
ansible-galaxy collection install community.general
ansible-galaxy collection install acme.tools:1.4.0 -s https://astrum.example.com/repository/ansible-group/
```

`requirements.yml` can name the repository of each collection:

```yaml
collections:
  - name: acme.tools
    version: ">=1.4.0"
    source: https://astrum.example.com/repository/ansible-group/
```

Publish an archive built with `ansible-galaxy collection build` to a hosted repository:

```shell
ansible-galaxy collection publish acme-tools-1.4.0.tar.gz \
  -s https://astrum.example.com/repository/ansible-hosted/ --token "$ASTRUM_TOKEN"
```

The archive must be named `<namespace>-<name>-<version>.tar.gz`, and its `MANIFEST.json` must name the same
collection with a semantic version; otherwise the import task that `ansible-galaxy` waits for fails and the
client prints the reason. The upload page and the components API take the same archives. The collection
list of a proxy (`api/v3/collections/`) shows the collections it has cached, not the whole remote. Roles
of the older Galaxy API (`ansible-galaxy role install`) are not served.
