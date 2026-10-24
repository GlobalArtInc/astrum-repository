# Composer

```shell
composer config repositories.composer-proxy composer https://astrum.example.com/repository/composer-proxy/
composer require monolog/monolog
```

Upload a package archive to a hosted repository:

```shell
curl -u "ci-deployer:$ASTRUM_PASSWORD" --upload-file package.zip \
  https://astrum.example.com/repository/composer-hosted/packages/upload/acme/tool/1.4.0
```
