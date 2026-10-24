# Maven

`~/.m2/settings.xml`:

```xml
<settings>
  <mirrors>
    <mirror>
      <id>astrum</id>
      <mirrorOf>*</mirrorOf>
      <url>https://astrum.example.com/repository/maven-public/</url>
    </mirror>
  </mirrors>
  <servers>
    <server>
      <id>astrum</id>
      <username>ci-deployer</username>
      <password>${env.ASTRUM_PASSWORD}</password>
    </server>
  </servers>
</settings>
```

Publishing, in the project's `pom.xml`:

```xml
<distributionManagement>
  <repository>
    <id>astrum</id>
    <url>https://astrum.example.com/repository/maven-releases/</url>
  </repository>
  <snapshotRepository>
    <id>astrum</id>
    <url>https://astrum.example.com/repository/maven-snapshots/</url>
  </snapshotRepository>
</distributionManagement>
```

## Gradle

```kotlin
repositories {
    maven {
        url = uri("https://astrum.example.com/repository/maven-public/")
        credentials {
            username = providers.environmentVariable("ASTRUM_USER").get()
            password = providers.environmentVariable("ASTRUM_PASSWORD").get()
        }
    }
}
```
