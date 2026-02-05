
package org.globalart.astrum.repository.content.fluent;

import java.util.Optional;

/**
 * Fluent API to create/find a component; at this point we already know the component name.
 *
 * @since 3.21
 */
public interface FluentComponentBuilder
{
  /**
   * Continue building the component using the given namespace.
   */
  FluentComponentBuilder namespace(String namespace);

  /**
   * Continue building the component using the given kind.
   *
   * @since 3.25
   */
  FluentComponentBuilder kind(String kind);

  /**
   * Set {@code kind} only if a value is present.
   *
   * @since 3.29
   */
  FluentComponentBuilder kind(Optional<String> optionalKind);

  /**
   * Continue building the component using the given version.
   */
  FluentComponentBuilder version(String version);

  /**
   * Continue building the component using the given normalized_version. Setting the version will also compute this
   * value and invoking this manually should not generally be required
   */
  FluentComponentBuilder normalizedVersion(String normalizedVersion);

  /**
   * Continue building the component using the given format attributes.
   *
   * @since 3.31
   */
  FluentComponentBuilder attributes(String key, Object value);

  /**
   * Gets the full component using the details built so far; if it doesn't exist then it is created.
   */
  FluentComponent getOrCreate();

  /**
   * Find if a component exists using the details built so far.
   */
  Optional<FluentComponent> find();

  /**
   * Find components within a group repository members
   *
   * @return stream of distinct components
   */
  Optional<FluentComponent> findInMembers();
}
