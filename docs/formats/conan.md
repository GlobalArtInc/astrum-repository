# Conan

The protocol version (Conan 1 or Conan 2) is chosen when the repository is created.

```shell
conan remote add conan-hosted https://astrum.example.com/repository/conan-hosted/
conan install --requires=zlib/1.3.1 -r conan-hosted
conan upload tool/1.4.0 -r conan-hosted
```
