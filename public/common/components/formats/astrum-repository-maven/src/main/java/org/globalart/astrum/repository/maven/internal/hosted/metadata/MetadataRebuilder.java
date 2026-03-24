
package org.globalart.astrum.repository.maven.internal.hosted.metadata;

import java.util.List;
import java.util.Set;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.transaction.Transactional;
import org.globalart.astrum.transaction.UnitOfWork;

public interface MetadataRebuilder
{
  /**
   * Rebuilds/updates Maven metadata.
   *
   * @param repository The repository whose metadata needs rebuild (Maven2 format, Hosted type only).
   * @param update if {@code true}, updates existing metadata, otherwise overwrites them with newly generated
   *          ones.
   * @param rebuildChecksums whether or not checksums should be checked and corrected if found
   *          missing or incorrect
   * @param cascadeUpdate should we rebuild nested levels if groupId/artifactId/baseVersion not present (empty)
   * @param groupId scope the work to given groupId.
   * @param artifactId scope the work to given artifactId (groupId must be given).
   * @param baseVersion scope the work to given baseVersion (groupId and artifactId must ge given).
   * @return whether the rebuild actually triggered
   */
  boolean rebuild(
      Repository repository,
      boolean update,
      boolean rebuildChecksums,
      boolean cascadeUpdate,
      @Nullable String groupId,
      @Nullable String artifactId,
      @Nullable String baseVersion);

  /**
   * Performs the {@link #rebuild} in an existing transactional context; ie. a {@link UnitOfWork} already exists.
   *
   * Don't put any {@link Transactional} annotation on this method because it will keep the transaction active
   * for the _entire_ duration of any request including those that rebuild the entire repository, which would
   * lead to excessive memory consumption. Callers may be annotated with {@link Transactional} as long as they
   * only rebuild a limited subset of the repository.
   */
  boolean rebuildInTransaction(
      Repository repository,
      boolean update,
      boolean rebuildChecksums,
      boolean cascadeUpdate,
      @Nullable String groupId,
      @Nullable String artifactId,
      @Nullable String baseVersion);

  /**
   * Delete the metadata for the input list of GAbVs.
   *
   * @param repository The repository whose metadata needs rebuilding (Maven2 format, Hosted type only).
   * @param gavs A list of gavs for which metadata will be deleted
   * @return The paths of the assets deleted
   *
   * @since 3.14
   */
  Set<String> deleteMetadata(Repository repository, List<String[]> gavs);
}
