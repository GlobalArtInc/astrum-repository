# NuGet

```shell
dotnet nuget add source https://astrum.example.com/repository/nuget-group/index.json \
  --name astrum --username ci-deployer --password "$ASTRUM_PASSWORD" --store-password-in-clear-text
```

For publishing activate the **NuGet API-Key Realm**, take the API key from the user menu
(**NuGet API Key**) and push:

```shell
dotnet nuget push MyLib.1.0.0.nupkg \
  --source https://astrum.example.com/repository/nuget-hosted/ --api-key "$NUGET_API_KEY"
```
