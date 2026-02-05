
package org.globalart.astrum.repository.content.fluent;

import java.util.Collection;
import java.util.Map;
import java.util.Optional;
import javax.annotation.Nullable;

import org.globalart.astrum.common.entity.Continuation;
import org.globalart.astrum.common.entity.EntityId;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.ComponentSet;
import org.globalart.astrum.repository.content.SqlGenerator;
import org.globalart.astrum.repository.content.SqlQueryParameters;
import org.globalart.astrum.repository.content.store.ComponentSetData;

/**
 * Fluent API for components.
 *
 * @since 3.21
 */
public interface FluentComponents
    extends FluentQuery<FluentComponent>
{
  /**
   * Start building a component, beginning with its name.
   */
  FluentComponentBuilder name(String name);

  /**
   * Interact with an existing component.
   */
  FluentComponent with(Component component);

  /**
   * Interact with an existing component and assets
   */
  FluentComponent with(Component component, @Nullable Collection<Asset> assets);

  /**
   * To include only group member content, ignoring components in the group repository itself. By defauly, only group
   * repository content is returned.
   */
  FluentQuery<FluentComponent> withOnlyGroupMemberContent();

  /**
   * To include both group content and group member content in the query. By default, only group
   * repository content is returned.
   */
  FluentQuery<FluentComponent> withGroupMemberContent();

  /**
   * Query components that have the given kind.
   */
  FluentQuery<FluentComponent> byKind(String kind);

  /**
   * Query components that match the given filter.
   * <p>
   * A filter parameter of {@code foo} should be referred to in the filter string as <code>#{filterParams.foo}</code>
   * <p>
   * <b>WARNING</b> the filter string is appended to the query and should only contain trusted content!
   */
  FluentQuery<FluentComponent> byFilter(String filter, Map<String, Object> filterParams);

  /**
   * Query components within the given set.
   */
  Continuation<FluentComponent> bySet(final ComponentSet componentSet, final int limit, final String continuationToken);

  /**
   * Select components using the provided query generator and parameters.
   *
   * @param generator generator for the select
   * @param params parameters for the select
   */
  Continuation<FluentComponent> selectComponents(
      final SqlGenerator<? extends SqlQueryParameters> generator,
      final SqlQueryParameters params);

  /**
   * Select components and related assets using the provided query generator and parameters.
   * 
   * @param generator generator for the select
   * @param params parameters for the select
   */
  Continuation<FluentComponent> selectComponentsWithAssets(
      final SqlGenerator<? extends SqlQueryParameters> generator,
      final SqlQueryParameters params);

  /**
   * List all namespaces of components in the repository.
   */
  Collection<String> namespaces();

  /**
   * List all names of components under the given namespace in the repository.
   */
  Collection<String> names(String namespace);

  /**
   * Browse through component sets in the repository.
   */
  Continuation<ComponentSetData> sets(int limit, @Nullable String continuationToken);

  /**
   * List all versions of components with the given namespace and name in the repository.
   */
  Collection<String> versions(String namespace, String name);

  /**
   * List all versions of components with the given namespace and name in for all the members in the group repository.
   */
  Collection<String> memberVersions(String namespace, String name);

  /**
   * Find if a component exists that has the given external id.
   */
  Optional<FluentComponent> find(EntityId externalId);

  /**
   * Browse components that have at least one associated asset.
   *
   * @param limit maximum number of components to return
   * @param continuationToken optional token to continue from a previous request
   * @return collection of components with assets
   */
  Continuation<FluentComponent> browseWithAssets(int limit, @Nullable String continuationToken);
}
