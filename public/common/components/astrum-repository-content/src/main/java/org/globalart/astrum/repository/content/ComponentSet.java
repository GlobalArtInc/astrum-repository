
package org.globalart.astrum.repository.content;

/**
 * Each component represents a unique logical coordinate in a repository.
 * Part of the coordinate is the namespace and name, which together form the component 'Set'
 * 
 * @see Component
 */
public interface ComponentSet
{
  /**
   * The component namespace; empty string if the component doesn't have a namespace.
   */
  String namespace();

  /**
   * The component name.
   */
  String name();

  default String toStringExternal() {
    return "namespace=" + namespace() +
        ", name=" + name();
  }
}
