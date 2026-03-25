
package org.globalart.astrum.repository.maven.internal.validation;

import java.io.IOException;
import java.io.InputStream;

import org.globalart.astrum.repository.InvalidContentException;
import org.globalart.astrum.repository.maven.MavenPath;
import org.globalart.astrum.repository.maven.internal.MavenModels;

import org.apache.maven.artifact.repository.metadata.Metadata;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static org.apache.commons.lang3.StringUtils.isNotEmpty;
import static org.globalart.astrum.repository.maven.internal.hosted.metadata.MetadataUtils.metadataPath;

/**
 * Validates that maven-metadata.xml is parsable and has the correct path.
 *
 * @since 3.16
 */
@Component
public class MavenMetadataContentValidator
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  public void validate(final String path, final InputStream mavenMetadata) {
    try {
      Metadata metadata = MavenModels.readMetadata(mavenMetadata);

      if (metadata == null) {
        throw new InvalidContentException("Metadata at path " + path + " is not a valid maven-metadata.xml");
      }

      // maven-metadata.xml files for plugins do not contain groupId and therefore cannot be validated
      if (isNotEmpty(metadata.getGroupId())) {
        validatePath(path, metadata);
      }
      else {
        log.debug("No groupId found in maven-metadata.xml therefore skipping validation");
      }
    }
    catch (IOException e) {
      log.warn("Unable to read maven-metadata.xml at path {}", path, e);

      throw new InvalidContentException("Unable to read maven-metadata.xml reason: " + e.getMessage());
    }
  }

  private void validatePath(final String path, final Metadata metadata) {
    String version = getMetadataVersion(metadata);

    MavenPath expectedPath = metadataPath(metadata.getGroupId(), metadata.getArtifactId(), version);

    if (!path.equals(expectedPath.getPath())) {
      String pattern = "Invalid maven-metadata.xml GAV %s, %s, %s does not match request path %s";

      String message = String
          .format(pattern, metadata.getGroupId(), metadata.getArtifactId(), metadata.getVersion(), path);

      log.warn("maven-metadata.xml path {} does not match the expected path {}", path, expectedPath.getPath());

      throw new InvalidContentException(message);
    }
  }

  private String getMetadataVersion(final Metadata metadata) {
    if (metadata.getVersion() != null && metadata.getVersion().contains("-SNAPSHOT")) {
      log.debug("maven-metadata.xml contains a SNAPSHOT version ({}) therefore the version is expected to be part of " +
          "the path", metadata.getVersion());

      return metadata.getVersion();
    }
    else {
      log.debug("maven-metadata.xml version ({}) is either null or not a SNAPSHOT therefore not expected in the path",
          metadata.getVersion());

      return null;
    }
  }
}
