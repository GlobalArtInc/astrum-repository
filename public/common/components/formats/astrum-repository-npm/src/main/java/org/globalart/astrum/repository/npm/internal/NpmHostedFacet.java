
package org.globalart.astrum.repository.npm.internal;

import java.io.IOException;
import java.util.Optional;
import java.util.Set;

import javax.annotation.Nullable;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Payload;

/**
 * npm hosted facet.
 *
 * @since 3.0
 */
@Facet.Exposed
public interface NpmHostedFacet
    extends Facet
{
  /**
   * Returns the package metadata.
   */
  Optional<Content> getPackage(NpmPackageId packageId) throws IOException;

  /**
   * Performs a "publish" of a package as sent by npm CLI.
   */
  void putPackage(NpmPackageId packageId, @Nullable String revision, Payload payload) throws IOException;

  /**
   * Deletes complete package along with all belonging tarballs too (if any).
   *
   * @return name of deleted asset(s).
   */
  Set<String> deletePackage(NpmPackageId packageId, @Nullable String revision) throws IOException;

  /**
   * Deletes complete package along with all belonging tarballs too (if any), maybe deletes the blobs.
   *
   * @return name of deleted asset(s).
   *
   * @since 3.9
   */
  Set<String> deletePackage(NpmPackageId packageId, @Nullable String revision, boolean deleteBlobs) throws IOException;

  /**
   * Returns the tarball content.
   */
  Optional<Content> getTarball(NpmPackageId packageId, String tarballName) throws IOException;

  /**
   * Deletes given tarball, if exists.
   *
   * @return name of deleted asset(s).
   */
  Optional<String> deleteTarball(NpmPackageId packageId, String tarballName) throws IOException;

  /**
   * Deletes given tarball, if exists, and maybe deletes the blob.
   *
   * @return name of deleted asset(s).
   *
   * @since 3.9
   */
  Optional<String> deleteTarball(NpmPackageId packageId, String tarballName, boolean deleteBlob) throws IOException;

  /**
   * Updates the package root.
   *
   * @param packageId
   * @param revision
   * @param newPackageRoot
   *
   * @since 3.10
   */
  void putPackageRoot(final NpmPackageId packageId, @Nullable final String revision,
                      final NestedAttributesMap newPackageRoot) throws IOException;

  /**
   * Returns the package metadata.
   */
  Optional<Content> getDistTags(NpmPackageId packageId) throws IOException;

  /**
   * Performs a "publish" of a dist-tag.
   */
  void putDistTags(NpmPackageId packageId, String tag, Payload payload) throws IOException;

  void deleteDistTags(NpmPackageId packageId, String tag, Payload payload) throws IOException;
}
