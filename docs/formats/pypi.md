# PyPI

`pip.conf` (or `PIP_INDEX_URL`):

```ini
[global]
index-url = https://astrum.example.com/repository/pypi-group/simple
```

Publishing with twine:

```shell
twine upload --repository-url https://astrum.example.com/repository/pypi-hosted/ \
  -u ci-deployer -p "$ASTRUM_PASSWORD" dist/*
```

uv and Poetry take the same URLs as an extra index.
