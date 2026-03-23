
package org.globalart.astrum.content.maven.internal.recipe;

import java.util.HashMap;
import java.util.Map;
import java.util.Optional;

import org.globalart.astrum.content.maven.store.Maven2ComponentData;
import org.globalart.astrum.content.maven.store.Maven2ComponentStore;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.maven.MavenPath;
import org.globalart.astrum.repository.maven.MavenPath.Coordinates;
import org.globalart.astrum.repository.maven.MavenPathParser;

import org.apache.maven.model.Model;

import static java.util.Optional.ofNullable;
import static org.globalart.astrum.repository.content.AttributeOperation.OVERLAY;
import static org.globalart.astrum.repository.maven.internal.Attributes.P_ARTIFACT_ID;
import static org.globalart.astrum.repository.maven.internal.Attributes.P_BASE_VERSION;
import static org.globalart.astrum.repository.maven.internal.Attributes.P_CLASSIFIER;
import static org.globalart.astrum.repository.maven.internal.Attributes.P_EXTENSION;
import static org.globalart.astrum.repository.maven.internal.Attributes.P_GROUP_ID;
import static org.globalart.astrum.repository.maven.internal.Attributes.P_PACKAGING;
import static org.globalart.astrum.repository.maven.internal.Attributes.P_POM_DESCRIPTION;
import static org.globalart.astrum.repository.maven.internal.Attributes.P_POM_NAME;
import static org.globalart.astrum.repository.maven.internal.Attributes.P_VERSION;
import static org.globalart.astrum.repository.maven.internal.Attributes.AssetKind.ARTIFACT;
import static org.globalart.astrum.repository.maven.internal.Attributes.AssetKind.ARTIFACT_SUBORDINATE;
import static org.globalart.astrum.repository.maven.internal.Attributes.AssetKind.OTHER;
import static org.globalart.astrum.repository.maven.internal.Attributes.AssetKind.REPOSITORY_INDEX;
import static org.globalart.astrum.repository.maven.internal.Attributes.AssetKind.REPOSITORY_METADATA;
import static org.globalart.astrum.repository.maven.internal.Maven2Format.NAME;

/**
 * Helper class used by {@link MavenContentFacetImpl} for setting Asset and Component attributes
 *
 * @since 3.26
 */
final class MavenAttributesHelper
{
  private static final String JAR = "jar";

  private static final int MAX_POM_FIELD_LENGTH = 10_000;

  private MavenAttributesHelper() {
    // no-op
  }

  static void setMavenAttributes(
      final Maven2ComponentStore componentStore,
      final FluentComponent component,
      final Coordinates coordinates,
      final Optional<Model> optionalModel,
      final int repositoryId)
  {
    Map<String, String> mavenAttributes = new HashMap<>();
    mavenAttributes.put(P_GROUP_ID, coordinates.getGroupId());
    mavenAttributes.put(P_ARTIFACT_ID, coordinates.getArtifactId());
    mavenAttributes.put(P_VERSION, coordinates.getVersion());
    mavenAttributes.put(P_BASE_VERSION, coordinates.getBaseVersion());
    mavenAttributes.put(P_EXTENSION, coordinates.getExtension());

    optionalModel.ifPresent(model -> {
      mavenAttributes.put(P_PACKAGING, getPackaging(model));
      ofNullable(model.getName())
          .map(MavenAttributesHelper::truncate)
          .ifPresent(name -> mavenAttributes.put(P_POM_NAME, name));
      ofNullable(model.getDescription())
          .map(MavenAttributesHelper::truncate)
          .ifPresent(desc -> mavenAttributes.put(P_POM_DESCRIPTION, desc));
    });

    component.attributes(OVERLAY, NAME, mavenAttributes);
    fillInBaseVersionColumn(componentStore, component, repositoryId, coordinates.getBaseVersion());
  }

  static void setMavenAttributes(final FluentAsset asset, final MavenPath mavenPath) {
    Map<String, String> mavenAttributes = new HashMap<>();
    Coordinates coordinates = mavenPath.getCoordinates();
    if (coordinates != null) {
      mavenAttributes.put(P_GROUP_ID, coordinates.getGroupId());
      mavenAttributes.put(P_ARTIFACT_ID, coordinates.getArtifactId());
      mavenAttributes.put(P_VERSION, coordinates.getVersion());
      mavenAttributes.put(P_BASE_VERSION, coordinates.getBaseVersion());
      ofNullable(coordinates.getClassifier()).ifPresent(value -> mavenAttributes.put(P_CLASSIFIER, value));
      mavenAttributes.put(P_EXTENSION, coordinates.getExtension());
    }
    asset.attributes(OVERLAY, NAME, mavenAttributes);
  }

  static String getPackaging(final Model model) {
    String packaging = model.getPackaging();
    return packaging == null ? JAR : packaging;
  }

  /**
   * Truncates the given value if it exceeds the maximum allowed length for POM metadata fields.
   * This prevents oversized strings from causing issues during JSON deserialization.
   */
  private static String truncate(final String value) {
    return value.length() > MAX_POM_FIELD_LENGTH
        ? value.substring(0, MAX_POM_FIELD_LENGTH)
        : value;
  }

  static String assetKind(final MavenPath mavenPath, final MavenPathParser mavenPathParser) {
    if (mavenPath.getCoordinates() != null) {
      return artifactRelatedAssetKind(mavenPath);
    }
    else {
      return fileAssetKindFor(mavenPath, mavenPathParser);
    }
  }

  private static String artifactRelatedAssetKind(final MavenPath mavenPath) {
    return mavenPath.isSubordinate() ? ARTIFACT_SUBORDINATE.name() : ARTIFACT.name();
  }

  private static String fileAssetKindFor(final MavenPath mavenPath, final MavenPathParser mavenPathParser) {
    if (mavenPathParser.isRepositoryMetadata(mavenPath)) {
      return REPOSITORY_METADATA.name();
    }
    else if (mavenPathParser.isRepositoryIndex(mavenPath)) {
      return REPOSITORY_INDEX.name();
    }
    else {
      return OTHER.name();
    }
  }

  /**
   * base_version column can be used to improve query speed and memory usage in some maven tasks.
   */
  private static void fillInBaseVersionColumn(
      final Maven2ComponentStore componentStore,
      final Component component,
      final int repositoryId,
      final String baseVersion)
  {
    Maven2ComponentData componentData = new Maven2ComponentData();
    componentData.setNamespace(component.namespace());
    componentData.setName(component.name());
    componentData.setVersion(component.version());
    componentData.setRepositoryId(repositoryId);
    componentData.setBaseVersion(baseVersion);
    componentStore.updateBaseVersion(componentData);
  }
}
