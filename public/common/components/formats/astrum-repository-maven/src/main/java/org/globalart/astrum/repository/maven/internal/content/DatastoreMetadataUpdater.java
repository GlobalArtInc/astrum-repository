
package org.globalart.astrum.repository.maven.internal.content;

import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.util.Map;
import java.util.Optional;

import org.globalart.astrum.content.maven.MavenContentFacet;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.AssetBlob;
import org.globalart.astrum.repository.maven.MavenPath;
import org.globalart.astrum.repository.maven.MavenPath.HashType;
import org.globalart.astrum.repository.maven.internal.Constants;
import org.globalart.astrum.repository.maven.internal.MavenMimeRulesSource;
import org.globalart.astrum.repository.maven.internal.MavenModels;
import org.globalart.astrum.repository.maven.internal.hosted.metadata.AbstractMetadataUpdater;
import org.globalart.astrum.repository.view.payloads.BytesPayload;
import org.globalart.astrum.repository.view.payloads.StringPayload;

import org.apache.maven.artifact.repository.metadata.Metadata;

import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.base.Preconditions.checkState;

/**
 * @since 3.26
 */
public class DatastoreMetadataUpdater
    extends AbstractMetadataUpdater
{
  public DatastoreMetadataUpdater(final boolean update, final Repository repository) {
    super(update, repository);
  }

  public DatastoreMetadataUpdater(
      final boolean update,
      final Repository repository,
      final boolean rebuildChecksums)
  {
    super(update, repository, rebuildChecksums);
  }

  /**
   * Checks if any checksum files are missing for the given metadata path.
   * <p>
   * This method checks for the existence of all four checksum types (SHA1, SHA256, SHA512, MD5).
   * If any checksum file is missing, it returns {@code true} to trigger regeneration.
   * <p>
   * <strong>Performance Note:</strong> This method performs up to 4 lightweight database lookups
   * (one per checksum type) to verify existence of each checksum asset. It does NOT fetch blob content,
   * making it efficient enough for use during metadata rebuild operations.
   *
   * @param mavenPath the metadata path to check
   * @return {@code true} if any checksum file is missing
   */
  @Override
  protected boolean hasMissingChecksums(final MavenPath mavenPath) {
    try {
      MavenContentFacet mavenContentFacet = repository.facet(MavenContentFacet.class);
      for (HashType hashType : HashType.values()) {
        MavenPath checksumPath = mavenPath.hash(hashType);
        if (!mavenContentFacet.exists(checksumPath)) {
          log.debug("Missing checksum file: {}", checksumPath.getPath());
          return true;
        }
      }
      return false;
    }
    catch (Exception e) {
      log.warn("Error checking for missing checksums for path: {}", mavenPath.getPath(), e);
      // If we can't determine, don't force a write - safe default
      return false;
    }
  }

  @Override
  protected void write(final MavenPath mavenPath, final Metadata metadata) throws IOException {
    MavenContentFacet mavenContentFacet = repository.facet(MavenContentFacet.class);
    final ByteArrayOutputStream buffer = new ByteArrayOutputStream();
    MavenModels.writeMetadata(buffer, metadata);
    mavenContentFacet.put(mavenPath, new BytesPayload(buffer.toByteArray(), MavenMimeRulesSource.METADATA_TYPE));

    final Optional<Map<String, String>> hashCodes = mavenContentFacet
        .assets()
        .path("/" + mavenPath.getPath())
        .find()
        .flatMap(Asset::blob)
        .map(AssetBlob::checksums);
    checkState(hashCodes.isPresent(), "hashCodes");

    for (HashType hashType : HashType.values()) {
      MavenPath checksumPath = mavenPath.hash(hashType);
      String hashCode = hashCodes.get().get(hashType.getHashAlgorithm().name());
      checkState(hashCode != null, "hashCode: type=%s", hashType);
      mavenContentFacet.put(checksumPath, new StringPayload(hashCode, Constants.CHECKSUM_CONTENT_TYPE));
    }
  }

  @Override
  protected Optional<Metadata> read(final MavenPath mavenPath) throws IOException {
    return repository.facet(MavenContentFacet.class)
        .get(mavenPath)
        .map(content -> {
          Metadata metadata = null;
          try {
            metadata = MavenModels.readMetadata(content.openInputStream());
          }
          catch (IOException e) {
            log.warn("Corrupted metadata {} @ {}", repository.getName(), mavenPath.getPath());
          }
          return metadata;
        });
  }

  @Override
  protected void delete(final MavenPath mavenPath) {
    checkNotNull(repository);
    checkNotNull(mavenPath);
    try {
      repository.facet(MavenContentFacet.class).deleteWithHashes(mavenPath);
    }
    catch (IOException e) {
      throw new RuntimeException(e);
    }
  }
}
