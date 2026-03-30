
package org.globalart.astrum.content.npm.internal;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;
import org.springframework.context.annotation.Scope;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;

import java.io.IOException;
import java.util.Collections;
import java.util.Optional;
import java.util.Set;
import java.util.stream.Stream;


import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.content.maintenance.ContentMaintenanceFacet;
import org.globalart.astrum.content.npm.NpmContentFacet;
import org.globalart.astrum.repository.npm.internal.NpmAttributes.AssetKind;
import org.globalart.astrum.repository.npm.internal.NpmHostedFacet;
import org.globalart.astrum.repository.npm.internal.NpmMetadataUtils;
import org.globalart.astrum.repository.npm.internal.NpmPackageId;
import org.globalart.astrum.repository.npm.internal.NpmPackageParser;

import com.google.common.collect.ImmutableSet;
import org.apache.commons.lang3.StringUtils;

import static java.util.Collections.*;

/**
 * This handles the deletes of Npm components and assets. A key piece of functionality is removing the dist tags for
 * version of packages from the root when they are removed.
 *
 * @since 3.29
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class NpmHostedMaintenanceFacet
    extends NpmFacetSupport
    implements ContentMaintenanceFacet
{
  @Autowired
  public NpmHostedMaintenanceFacet(final NpmPackageParser npmPackageParser) {
    super(npmPackageParser);
  }

  @Override
  public Set<String> deleteComponent(final org.globalart.astrum.repository.content.Component component) {
    log.debug("Deleting npm component: {}", component);

    ImmutableSet.Builder<String> deletedAssets = ImmutableSet.builder();

    FluentComponent fluentComponent = getRepository()
        .facet(ContentFacet.class)
        .components()
        .with(component);

    fluentComponent.assets().forEach(asset -> deletedAssets.addAll(deleteAsset(asset)));

    return deletedAssets.build();
  }

  @Override
  public Set<String> deleteAsset(final Asset asset) {
    log.debug("Deleting NPM asset: {}", asset);

    AssetKind assetKind = AssetKind.valueOf(asset.kind());
    ImmutableSet.Builder<String> deletedAssets = ImmutableSet.builder();

    switch (assetKind) {
      case PACKAGE_ROOT:
        deletedAssets.addAll(deletePackageRoot(asset));
        break;
      case TARBALL:
        try {
          deletedAssets.addAll(deleteTarball(asset));
        }
        catch (IOException e) {
          log.error("Exception deleting tarball for asset: {}", asset, e);
          return emptySet();
        }
        break;
      default:
        break;
    }
    return deletedAssets.build();
  }

  @Override
  public int deleteComponents(final Stream<FluentComponent> components) {
    return (int) components.map(this::deleteComponent).count();
  }

  /**
   * Deletes package and all related tarballs too.
   */
  private Set<String> deletePackageRoot(final Asset asset) {
    NpmPackageId packageId = NpmPackageId.parse(removeStartingSlash(asset.path()));
    try {
      return getRepository().facet(NpmHostedFacet.class).deletePackage(packageId, null);
    }
    catch (IOException e) {
      log.error("Exception delete package root for asset: {}", asset, e);
      return emptySet();
    }
  }

  private Set<String> deleteTarball(final Asset asset) throws IOException
  {
    log.debug("Deleting tarball for asset: {}", asset);

    String assetNameWithoutStartingSlash = removeStartingSlash(asset.path());
    NpmPackageId packageId =
        NpmPackageId.parse(assetNameWithoutStartingSlash.substring(0, assetNameWithoutStartingSlash.indexOf("/-/")));
    String tarballName = NpmMetadataUtils.extractTarballName(assetNameWithoutStartingSlash);

    Optional<FluentAsset> packageRootAsset = findPackageRootAsset(packageId);
    if (!packageRootAsset.isPresent()) {
      log.debug("No package root present for asset with path: {}", asset.path());
      return emptySet();
    }

    Optional<NestedAttributesMap> nestedAttributesMapOptional =
        loadPackageRoot(packageId, getRepository().facet(NpmContentFacet.class));

    if (!nestedAttributesMapOptional.isPresent()) {
      log.debug("No attributes for package root present for asset with path: {}", asset.path());
      return emptySet();
    }
    NestedAttributesMap packageRoot = nestedAttributesMapOptional.get();

    NestedAttributesMap version = NpmMetadataUtils.selectVersionByTarballName(packageRoot, tarballName);

    if (version == null) {
      return emptySet();
    }

    packageRoot.child(NpmMetadataUtils.VERSIONS).remove(version.getKey());
    if (packageRoot.child(NpmMetadataUtils.VERSIONS).isEmpty()) {
      return getRepository().facet(NpmHostedFacet.class).deletePackage(packageId, null);
    }
    else {
      log.debug("Removing dist tags for asset {} with version {}", asset.path(), version.getKey());
      removeDistTagsFromTagsWithVersion(packageRoot, version.getKey());

      packageRoot.child(NpmMetadataUtils.TIME).remove(version.getKey());
      NpmMetadataUtils.maintainTime(packageRoot);
      savePackageRoot(packageId, packageRoot);
      return getRepository().facet(NpmHostedFacet.class)
          .deleteTarball(packageId, tarballName)
          .map(Collections::singleton)
          .orElseGet(Collections::emptySet);
    }
  }

  private String removeStartingSlash(final String toRemoveFrom) {
    return StringUtils.removeStart(toRemoveFrom, "/");
  }
}
