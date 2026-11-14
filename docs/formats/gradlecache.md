# Gradle build cache

Hosted `gradlecache` repositories are a self-hosted [Gradle build cache](https://docs.gradle.org/current/userguide/build_cache.html):
Gradle uploads the outputs of its tasks and restores them on other machines and in CI instead of running the tasks again.
The server answers the HTTP build cache protocol of Gradle: `GET` and `PUT` on `<repository>/<key>`, where the key is the
hexadecimal cache key that Gradle computes; the format has no proxy or group repositories.

Enable the cache for every build in `gradle.properties`:

```properties
org.gradle.caching=true
```

and point Gradle at the repository in `settings.gradle.kts`:

```kotlin
buildCache {
    remote<HttpBuildCache> {
        url = uri("https://astrum.example.com/repository/gradle-cache/")
        isPush = true
        credentials {
            username = "<user>"
            password = "<access token>"
        }
    }
}
```

CI passes `--build-cache` to Gradle and uploads with the same block, so that the outputs it builds reach the cache:

```shell
./gradlew build --build-cache
```

Gradle sends the credentials as HTTP Basic. The password is an [access token](../configure/authentication.md#access-tokens),
and the user name must be the name of the user who owns the token; any other name, such as a made-up one, answers 401.
Use a token with only the rights that the machine needs: a read token for developers
restores outputs, a write token for CI also uploads them. A token that may not write gets a 403 on upload, and Gradle
counts that answer as a failed upload.

A stored entry is never replaced: a second upload of the same key answers 200 and keeps the first entry, because the
keys are computed from the inputs of the task. Keys must be 8 to 128 hexadecimal characters; any other key is refused
with 400 on upload and 404 on download. Each entry is stored as `<repository>/<key>` in lower case and can be downloaded
there, which is what the repository browser links to. An administrator can delete an entry, and the key can then be
uploaded again; old entries go away with a cleanup policy on the time they were last downloaded.
