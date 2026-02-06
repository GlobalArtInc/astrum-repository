
package org.globalart.astrum.repository.content.store;

import java.util.List;
import javax.annotation.Nullable;

import org.globalart.astrum.common.entity.ContinuationAware;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.Component;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * {@link Component} data backed by the content data store.
 */
public class ComponentData
    extends AbstractRepositoryContent
    implements Component, ContinuationAware
{
  Integer componentId; // NOSONAR: internal id

  private String namespace;

  private String name;

  private String kind;

  private String version;

  private String normalizedVersion;

  private Integer entityVersion;

  private List<Asset> assets;

  public ComponentData() {
    // empty constructor to be able to instantiate object using myBatis
  }

  // Component API

  @Override
  public String namespace() {
    return namespace;
  }

  @Override
  public String name() {
    return name;
  }

  @Override
  public String kind() {
    return kind;
  }

  @Override
  public String version() {
    return version;
  }

  @Override
  public String normalizedVersion() {
    return normalizedVersion;
  }

  @Override
  public Integer entityVersion() {
    return entityVersion;
  }

  // MyBatis setters + validation

  /**
   * Sets the internal component id.
   */
  public void setComponentId(final int componentId) {
    this.componentId = componentId;
  }

  public Integer componentId() {
    return componentId;
  }

  /**
   * Sets the component namespace.
   */
  public void setNamespace(final String namespace) {
    this.namespace = checkNotNull(namespace);
  }

  /**
   * Sets the component name.
   */
  public void setName(final String name) {
    this.name = checkNotNull(name);
  }

  /**
   * Sets the component kind.
   *
   * @since 3.25
   */
  public void setKind(final String kind) {
    this.kind = checkNotNull(kind);
  }

  /**
   * Sets the component version.
   */
  public void setVersion(final String version) {
    this.version = checkNotNull(version);
  }

  /**
   * Sets the component normalized version
   */
  public void setNormalizedVersion(final String normalizedVersion) {
    this.normalizedVersion = checkNotNull(normalizedVersion);
  }

  /**
   * Sets the entity version.
   */
  public void setEntityVersion(final Integer entityVersion) {
    this.entityVersion = entityVersion;
  }

  @Nullable
  public List<Asset> getAssets() {
    return assets;
  }

  public void setAssets(final List<Asset> assets) {
    this.assets = assets;
  }

  // ContinuationAware

  @Override
  public String nextContinuationToken() {
    return Integer.toString(componentId);
  }

  @Override
  public String toString() {
    return "ComponentData{" +
        "componentId=" + componentId +
        ", namespace='" + namespace + '\'' +
        ", name='" + name + '\'' +
        ", kind='" + kind + '\'' +
        ", version='" + version + '\'' +
        ", normalizedVersion='" + normalizedVersion + '\'' +
        ", entityVersion='" + entityVersion + '\'' +
        "} " + super.toString();
  }
}
