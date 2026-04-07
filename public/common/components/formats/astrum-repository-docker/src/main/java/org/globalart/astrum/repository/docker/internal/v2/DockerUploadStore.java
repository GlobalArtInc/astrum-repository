package org.globalart.astrum.repository.docker.internal.v2;

import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.io.UncheckedIOException;
import java.nio.file.DirectoryStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardOpenOption;
import java.nio.file.attribute.FileTime;
import java.time.Duration;
import java.time.Instant;
import java.util.Optional;
import java.util.UUID;
import java.util.concurrent.atomic.AtomicLong;
import java.util.regex.Pattern;

import org.globalart.astrum.bootstrap.entrypoint.configuration.ApplicationDirectories;
import org.globalart.astrum.repository.Repository;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

@Component
public class DockerUploadStore
{
  private static final Logger log = LoggerFactory.getLogger(DockerUploadStore.class);

  private static final Pattern SAFE_SEGMENT = Pattern.compile("^[a-zA-Z0-9._-]+$");

  private static final long PURGE_INTERVAL_MILLIS = Duration.ofMinutes(10).toMillis();

  private final Path root;

  private final Duration maxAge;

  private final AtomicLong lastPurge = new AtomicLong();

  @Autowired
  public DockerUploadStore(
      final ApplicationDirectories applicationDirectories,
      @Value("${astrum.docker.upload.maxAgeHours:24}") final int maxAgeHours)
  {
    this.root = applicationDirectories.getTemporaryDirectory().toPath().resolve("docker-uploads");
    this.maxAge = Duration.ofHours(maxAgeHours);
  }

  public String start(final Repository repository) throws IOException {
    purgeExpired();
    String uuid = UUID.randomUUID().toString();
    Path file = path(repository, uuid);
    Files.createDirectories(file.getParent());
    Files.createFile(file);
    return uuid;
  }

  public Optional<Path> find(final Repository repository, final String uuid) {
    if (!SAFE_SEGMENT.matcher(uuid).matches()) {
      return Optional.empty();
    }
    Path file = path(repository, uuid);
    return Files.isRegularFile(file) ? Optional.of(file) : Optional.empty();
  }

  public long size(final Path file) {
    try {
      return Files.size(file);
    }
    catch (IOException e) {
      throw new UncheckedIOException(e);
    }
  }

  public long append(final Path file, final InputStream in) throws IOException {
    try (OutputStream out = Files.newOutputStream(file, StandardOpenOption.WRITE, StandardOpenOption.APPEND)) {
      in.transferTo(out);
    }
    Files.setLastModifiedTime(file, FileTime.from(Instant.now()));
    return Files.size(file);
  }

  public void delete(final Path file) {
    try {
      Files.deleteIfExists(file);
    }
    catch (IOException e) {
      log.warn("Unable to delete docker upload {}", file, e);
    }
  }

  private Path path(final Repository repository, final String uuid) {
    String repositoryName = repository.getName();
    String directory = SAFE_SEGMENT.matcher(repositoryName).matches()
        ? repositoryName
        : Integer.toHexString(repositoryName.hashCode());
    return root.resolve(directory).resolve(uuid);
  }

  private void purgeExpired() {
    long now = System.currentTimeMillis();
    long previous = lastPurge.get();
    if (now - previous < PURGE_INTERVAL_MILLIS || !lastPurge.compareAndSet(previous, now)) {
      return;
    }
    if (!Files.isDirectory(root)) {
      return;
    }
    Instant cutoff = Instant.now().minus(maxAge);
    try (DirectoryStream<Path> repositories = Files.newDirectoryStream(root)) {
      for (Path repositoryDirectory : repositories) {
        if (!Files.isDirectory(repositoryDirectory)) {
          continue;
        }
        try (DirectoryStream<Path> uploads = Files.newDirectoryStream(repositoryDirectory)) {
          for (Path upload : uploads) {
            if (Files.getLastModifiedTime(upload).toInstant().isBefore(cutoff)) {
              log.debug("Purging abandoned docker upload {}", upload);
              delete(upload);
            }
          }
        }
      }
    }
    catch (IOException e) {
      log.warn("Unable to purge abandoned docker uploads in {}", root, e);
    }
  }
}
