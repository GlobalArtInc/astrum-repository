
package org.globalart.astrum.repository.content.fluent.internal;

import java.util.List;
import java.util.Map;

import javax.annotation.Nullable;

import org.globalart.astrum.common.entity.Continuation;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.fluent.FluentQuery;
import org.globalart.astrum.repository.content.fluent.constraints.FluentQueryConstraint;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.util.Collections.emptyList;

/**
 * {@link FluentQuery} implementation for {@link FluentAsset}s.
 *
 * @since 3.26
 */
public class FluentAssetQueryImpl
    implements FluentQuery<FluentAsset>
{
  private final FluentAssetsImpl assets;

  private final String kind;

  private final String filter;

  private final Map<String, Object> filterParams;

  private final List<FluentQueryConstraint> constraints;

  FluentAssetQueryImpl(final FluentAssetsImpl assets, final List<FluentQueryConstraint> constraints) {
    this.assets = checkNotNull(assets);
    this.constraints = checkNotNull(constraints);
    this.filter = null;
    this.filterParams = null;
    this.kind = null;
  }

  FluentAssetQueryImpl(final FluentAssetsImpl assets, final String kind) {
    this.assets = checkNotNull(assets);
    this.kind = checkNotNull(kind);
    this.filter = null;
    this.filterParams = null;
    this.constraints = emptyList();
  }

  FluentAssetQueryImpl(
      final FluentAssetsImpl assets,
      final String filter,
      final Map<String, Object> filterParams)
  {
    this.assets = checkNotNull(assets);
    this.kind = null;
    this.filter = checkNotNull(filter);
    this.filterParams = checkNotNull(filterParams);
    this.constraints = emptyList();
  }

  @Override
  public int count() {
    return assets.doCount(kind, filter, filterParams);
  }

  @Override
  public Continuation<FluentAsset> browse(final int limit, final String continuationToken) {
    return assets.doBrowse(limit, continuationToken, kind, filter, filterParams, constraints);
  }

  @Override
  public Continuation<FluentAsset> browseEager(final int limit, @Nullable final String continuationToken) {
    throw new UnsupportedOperationException();
  }
}
