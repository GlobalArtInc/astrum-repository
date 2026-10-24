# Helm

```shell
helm repo add astrum https://astrum.example.com/repository/helm-group/ \
  --username ci-deployer --password "$ASTRUM_PASSWORD"
helm repo update
curl -u "ci-deployer:$ASTRUM_PASSWORD" --upload-file mychart-1.2.0.tgz \
  https://astrum.example.com/repository/helm-hosted/
```
