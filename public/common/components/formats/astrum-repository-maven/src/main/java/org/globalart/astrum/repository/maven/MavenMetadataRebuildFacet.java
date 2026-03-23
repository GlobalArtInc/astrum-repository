
package org.globalart.astrum.repository.maven;

import java.io.IOException;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.Facet;

/**
 * @since 3.29
 */
@Facet.Exposed
public interface MavenMetadataRebuildFacet
    extends Facet
{
  String METADATA_FORCE_REBUILD = "forceRebuild";

  String METADATA_REBUILD = "metadataRebuild";

  /**
   * Rebuilds/updates Maven metadata. The parameters depend each on previous, and if any of those are set (ie. G, GA or
   * GAV), the metadata will be updated. Rebuild is possible only against repository as whole, not a sub-part of it.
   *
   * @param groupId scope the work to given groupId.
   * @param artifactId scope the work to given artifactId (groupId must be given).
   * @param baseVersion scope the work to given baseVersion (groupId and artifactId must be given).
   * @param rebuildChecksums whether or not checksums should be checked and corrected if found
   *          missing or incorrect
   */
  void rebuildMetadata(
      @Nullable String groupId,
      @Nullable String artifactId,
      @Nullable String baseVersion,
      boolean rebuildChecksums);

  /**
   * Rebuilds/updates Maven metadata. The parameters depend each on previous, and if any of those are set (ie. G, GA or
   * GAV), the metadata will be updated. Rebuild is possible only against repository as whole, not a sub-part of it.
   *
   * @param groupId scope the work to given groupId.
   * @param artifactId scope the work to given artifactId (groupId must be given).
   * @param baseVersion scope the work to given baseVersion (groupId and artifactId must be given).
   * @param rebuildChecksums whether or not checksums should be checked and corrected if found
   *          missing or incorrect
   * @param update whether to update or replace metadata
   *
   * @since 3.22
   */
  void rebuildMetadata(
      @Nullable String groupId,
      @Nullable String artifactId,
      @Nullable String baseVersion,
      boolean rebuildChecksums,
      boolean update);

  /**
   * Rebuilds/updates Maven metadata. The parameters depend each on previous, and if any of those are set (ie. G, GA or
   * GAV), the metadata will be updated. Rebuild is possible only against repository as whole, not a sub-part of it.
   *
   * @param groupId scope the work to given groupId.
   * @param artifactId scope the work to given artifactId (groupId must be given).
   * @param baseVersion scope the work to given baseVersion (groupId and artifactId must be given).
   * @param rebuildChecksums whether or not checksums should be checked and corrected if found
   *          missing or incorrect
   * @param cascadeUpdate should we rebuild nested levels if groupId/artifactId/baseVersion not present (empty)
   * @param update whether to update or replace metadata
   *
   */
  void rebuildMetadata(
      @Nullable String groupId,
      @Nullable String artifactId,
      @Nullable String baseVersion,
      boolean rebuildChecksums,
      boolean cascadeUpdate,
      boolean update);

  /**
   * Rebuilds/updates Maven metadata if an asset is available at {@code path} which has an associated
   * blob.
   *
   * @param path
   * @param update
   * @param rebuildChecksums
   * @throws IOException
   *
   * @since 3.30
   */
  default void maybeRebuildMavenMetadata(
      String path,
      boolean update,
      boolean rebuildChecksums) throws IOException
  {
    // do nothing by default
  }
}
