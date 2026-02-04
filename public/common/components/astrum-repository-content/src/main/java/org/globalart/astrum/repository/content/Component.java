
package org.globalart.astrum.repository.content;

/**
 * Each component represents a unique logical coordinate in a repository.
 *
 * @see Asset
 */
public interface Component
    extends RepositoryContent
{
  /**
   * The component namespace; empty string if the component doesn't have a namespace.
   */
  String namespace();

  /**
   * The component name.
   */
  String name();

  /**
   * The component kind.
   *
   * @since 3.25
   */
  String kind();

  /**
   * The component version; empty string if the component doesn't have a version.
   */
  String version();

  /**
   * The component normalized version; if null then normalizeVersion task is in progress
   */
  String normalizedVersion();

  /**
   * The entity version
   */
  Integer entityVersion();

  default String toStringExternal() {
    return "namespace=" + namespace() +
        ", name=" + name() +
        ", version=" + version();
  }
}
