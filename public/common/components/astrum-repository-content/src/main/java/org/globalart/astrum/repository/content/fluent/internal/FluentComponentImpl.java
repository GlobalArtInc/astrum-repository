
package org.globalart.astrum.repository.content.fluent.internal;

import java.time.OffsetDateTime;
import java.util.Collection;
import java.util.Collections;
import java.util.Objects;

import javax.annotation.Nullable;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.AttributeOperation;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.facet.ContentFacetSupport;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.fluent.FluentAssetBuilder;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.content.store.ComponentData;
import org.globalart.astrum.repository.content.store.WrappedContent;

import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.collect.Collections2.transform;

/**
 * {@link FluentComponent} implementation.
 *
 * @since 3.24
 */
public class FluentComponentImpl
    implements FluentComponent, WrappedContent<Component>
{
  private final ContentFacetSupport facet;

  private final Component component;

  private final Collection<FluentAsset> assets;

  public FluentComponentImpl(final ContentFacetSupport facet, final Component component) {
    this(facet, component, null);
  }

  public FluentComponentImpl(
      final ContentFacetSupport facet,
      final Component component,
      @Nullable final Collection<FluentAsset> assets)
  {
    this.facet = checkNotNull(facet);
    this.component = checkNotNull(component);
    this.assets = assets == null ? null : Collections.unmodifiableCollection(assets);
  }

  @Override
  public Repository repository() {
    return facet.repository();
  }

  @Override
  public String namespace() {
    return component.namespace();
  }

  @Override
  public String name() {
    return component.name();
  }

  @Override
  public String kind() {
    return component.kind();
  }

  @Override
  public String version() {
    return component.version();
  }

  @Override
  public String normalizedVersion() {
    return component.normalizedVersion();
  }

  @Override
  public Integer entityVersion() {
    return component.entityVersion();
  }

  @Override
  public NestedAttributesMap attributes() {
    return component.attributes();
  }

  @Override
  public OffsetDateTime created() {
    return component.created();
  }

  @Override
  public OffsetDateTime lastUpdated() {
    return component.lastUpdated();
  }

  @Override
  public FluentComponent attributes(final AttributeOperation change, final String key, final Object value) {
    facet.stores().componentStore.updateComponentAttributes(component, change, key, value);
    return this;
  }

  @Override
  public FluentAssetBuilder asset(final String path) {
    return new FluentAssetBuilderImpl(facet, facet.stores().assetStore, path).component(this);
  }

  @Override
  public Collection<FluentAsset> assets() {
    if (assets != null) {
      return assets;
    }

    return transform(facet.stores().assetStore().browseComponentAssets(component),
        asset -> new FluentAssetImpl(facet, asset));
  }

  @Override
  public Collection<FluentAsset> assets(boolean useCache) {
    if (useCache && component instanceof ComponentData && ((ComponentData) component).getAssets() != null) {
      return transform(((ComponentData) component).getAssets(), asset -> new FluentAssetImpl(facet, asset));
    }
    return assets();
  }

  @Override
  public FluentComponent kind(final String kind) {
    if (!Objects.equals(kind, component.kind())) {
      ((ComponentData) component).setKind(kind);
      facet.stores().componentStore.updateComponentKind(component);
    }
    return this;
  }

  @Override
  public boolean delete() {
    return facet.stores().componentStore.deleteComponent(component);
  }

  @Override
  public Component unwrap() {
    return component;
  }

  @Override
  public String toString() {
    return toStringExternal();
  }
}
