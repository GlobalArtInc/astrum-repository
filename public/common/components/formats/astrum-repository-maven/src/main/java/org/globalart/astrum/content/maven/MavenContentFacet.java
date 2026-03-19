
package org.globalart.astrum.content.maven;

import java.io.IOException;
import java.util.Collection;
import java.util.List;
import java.util.Optional;
import java.util.Set;
import java.util.stream.Stream;
import javax.annotation.Nullable;

import org.globalart.astrum.common.entity.Continuation;
import org.globalart.astrum.content.maven.store.GAV;
import org.globalart.astrum.content.maven.store.Maven2ComponentData;
import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.maven.MavenFacet;
import org.globalart.astrum.repository.maven.MavenPath;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Payload;

/**
 * Provides persistence operations for Maven.
 *
 * @since 3.25
 */
@Facet.Exposed
public interface MavenContentFacet
    extends ContentFacet, MavenFacet
{
  /**
   * Get a maven asset
   *
   * @param mavenPath Path of asset to get
   */
  Optional<Content> get(MavenPath mavenPath) throws IOException;

  /**
   * Check if a maven asset exists
   *
   * @param mavenPath Path of asset to check
   * @return true if asset exists, false otherwise
   */
  boolean exists(MavenPath mavenPath);

  /**
   * Put a maven asset
   *
   * @param path The path of the asset to put
   * @param content The content to put
   */
  Content put(MavenPath path, Payload content) throws IOException;

  /**
   * Delete a maven asset
   *
   * @param path The path of the asset to delete.
   * @return True if asset was deleted or false if not.
   */
  boolean delete(MavenPath path) throws IOException;

  /**
   * Deletes the assets at the specified paths.
   *
   * @since 3.29
   */
  boolean delete(final List<String> paths);

  /**
   * Delete the specified maven asset and its hashes
   *
   * @param path The path of the asset to delete
   * @return The paths of the assets deleted
   */
  Set<String> deleteWithHashes(MavenPath path) throws IOException;

  /**
   * Update component attributes when {@code path} corresponds to a Maven POM
   *
   * @param path for which the corresponding component attributes may be updated
   */
  void maybeUpdateComponentAttributes(MavenPath path) throws IOException;

  int deleteComponents(int[] componentIds);

  int deleteComponents(Stream<FluentComponent> components);

  /**
   * Delete metadata, when no more components of same coordinates; or Flag metadata for rebuild, when other components
   * share same GAbV (snapshots)
   *
   * @param component for which metadata should be deleted or flagged for rebuild
   * @return paths of deleted assets; empty when just flagging for rebuild
   */
  Set<String> deleteMetadataOrFlagForRebuild(Component component);

  /**
   * Find Snapshot Group Artifact Version(GAVs)
   *
   * @param minimumRetained The minimum number of snapshots to keep.
   *
   * @since 3.30
   */
  Set<GAV> findGavsWithSnaphots(int minimumRetained);

  /**
   * Find Components by Group Artifact Version(GAVs)
   * Eagerly fetches {@link org.globalart.astrum.repository.content.store.AssetBlobData}
   * & {@link org.globalart.astrum.repository.content.store.AssetData}
   * 
   * @param name artifact name
   * @param group artifact group
   * @param baseVersion artifact base version
   * @param releaseVersion artifact release version
   *
   * @since 3.30
   */
  List<Maven2ComponentData> findComponentsForGav(
      final String name,
      final String group,
      final String baseVersion,
      final String releaseVersion);

  /**
   * Find jar assets associated with Components in the namespace of kind maven-plugin.
   *
   * @param limit maximum number of assets to return
   * @param continuationToken optional token to continue from a previous request
   * @param namespace the namespace to find plugins for
   *
   * @return collection of assets and the next continuation token
   *
   * @see Continuation#nextContinuationToken()
   */
  Continuation<Asset> findMavenPluginAssetsForNamespace(
      int limit,
      @Nullable String continuationToken,
      String namespace);

  /**
   * Find components with the GAbV.
   *
   * @param limit maximum number of assets to return
   * @param continuationToken optional token to continue from a previous request
   * @param namespace the namespace
   * @param name artifact name
   * @param baseVersion artifact base version
   *
   * @return collection of assets and the next continuation token
   *
   * @see Continuation#nextContinuationToken()
   */
  Continuation<FluentComponent> findComponentsForBaseVersion(
      int limit,
      @Nullable String continuationToken,
      String namespace,
      String name,
      String baseVersion);

  /**
   * Find Components in the provided GroupId & ArtifactId.
   *
   * @param limit maximum number of assets to return
   * @param continuationToken optional token to continue from a previous request
   * @param namespace the namespace
   * @param name artifact name
   *
   * @return collection of assets and the next continuation token
   *
   * @see Continuation#nextContinuationToken()
   */
  Continuation<FluentComponent> findComponentsInGA(
      int limit,
      @Nullable String continuationToken,
      String namespace,
      String name);

  /**
   * Retrieve known base versions for a provided GA.
   *
   * @param namespace the namespace for the components
   * @param name the name for the components
   *
   * @return a unique set of base versions
   */
  Collection<String> getBaseVersions(String namespace, String name);

  /**
   * Find components with missed base version.
   *
   * @param namespace the namespace
   * @param name artifact name
   *
   * @return collection of component data
   */
  public Iterable<FluentComponent> getComponentsWithMissedBaseVersion();

  /**
   * Updates the maven base_version of the given component in the content data store.
   *
   * @param component the component to update
   */
  public void updateBaseVersion(final Maven2ComponentData component);

  /**
   * Find snapshots to delete for which a release version exists
   *
   * @param gracePeriod an optional period to keep snapshots around.
   * @return array of snapshot components IDs to delete for which a release version exists
   *
   * @since 3.30
   */
  int[] selectSnapshotsAfterRelease(final int gracePeriod);

  /**
   * Create a component and asset for a maven path without attaching a blob to the asset. This is primarily used when
   * the blob will be hard linked to the asset afterwards.
   *
   * @param mavenPath
   * @return the asset
   */
  FluentAsset createComponentAndAsset(final MavenPath mavenPath);

  /**
   * @param source component to copy
   * @return a new component copied from the source to the current repository
   *
   * @since 3.38
   */
  FluentComponent copy(final Component source);
}
