
package org.globalart.astrum.repository.content.fluent.internal;

import java.util.List;
import java.util.Map;
import javax.annotation.Nullable;

import org.globalart.astrum.common.entity.Continuation;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.content.fluent.FluentQuery;
import org.globalart.astrum.repository.content.fluent.constraints.FluentQueryConstraint;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.util.Collections.emptyList;

/**
 * {@link FluentQuery} implementation for {@link FluentComponent}s.
 *
 * @since 3.26
 */
public class FluentComponentQueryImpl
    implements FluentQuery<FluentComponent>
{
  private final FluentComponentsImpl components;

  private final String kind;

  private final String filter;

  private final Map<String, Object> filterParams;

  private final List<FluentQueryConstraint> constraints;

  FluentComponentQueryImpl(final FluentComponentsImpl components, final List<FluentQueryConstraint> constraints) {
    this.components = checkNotNull(components);
    this.constraints = checkNotNull(constraints);
    this.filter = null;
    this.filterParams = null;
    this.kind = null;
  }

  FluentComponentQueryImpl(final FluentComponentsImpl components, final String kind) {
    this.components = checkNotNull(components);
    this.kind = checkNotNull(kind);
    this.filter = null;
    this.filterParams = null;
    this.constraints = emptyList();
  }

  FluentComponentQueryImpl(
      final FluentComponentsImpl components,
      final String filter,
      final Map<String, Object> filterParams)
  {
    this.components = checkNotNull(components);
    this.kind = null;
    this.filter = checkNotNull(filter);
    this.filterParams = checkNotNull(filterParams);
    this.constraints = emptyList();
  }

  @Override
  public int count() {
    return components.doCount(kind, filter, filterParams);
  }

  @Override
  public Continuation<FluentComponent> browse(final int limit, final String continuationToken) {
    return components.doBrowse(limit, continuationToken, kind, filter, filterParams, constraints);
  }

  @Override
  public Continuation<FluentComponent> browseEager(final int limit, @Nullable final String continuationToken) {
    return components.doBrowseEager(limit, continuationToken, kind, filter, filterParams);
  }
}
