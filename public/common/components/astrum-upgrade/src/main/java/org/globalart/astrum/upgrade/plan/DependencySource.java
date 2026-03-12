
package org.globalart.astrum.upgrade.plan;

import java.util.Collection;
import java.util.List;

/**
 * {@link Dependency} source.
 *
 * @since 3.1
 */
public interface DependencySource<T>
{
  /**
   * Returns a list of dependencies.
   */
  List<Dependency<T>> getDependencies();

  /**
   * Allow {@link DependencySource} to become aware of what it depends on.
   */
  interface DependsOnAware<T>
  {
    void setDependsOn(Collection<T> dependsOn);
  }

  /**
   * Allow {@link DependencySource} to become aware of unresolved dependencies.
   */
  interface UnresolvedDependencyAware<T>
  {
    void setUnresolvedDependencies(Collection<Dependency<T>> unresolved) throws Exception;
  }
}
