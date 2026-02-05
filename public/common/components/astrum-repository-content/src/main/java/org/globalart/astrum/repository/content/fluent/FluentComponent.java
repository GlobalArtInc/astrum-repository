
package org.globalart.astrum.repository.content.fluent;

import java.util.Collection;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.Component;

/**
 * Fluent API for a particular component.
 *
 * @since 3.21
 */
public interface FluentComponent
    extends Component, FluentAttributes<FluentComponent>
{
  /**
   * The repository containing this component.
   *
   * @since 3.24
   */
  Repository repository();

  /**
   * Start building an asset for this component, beginning with its path.
   */
  FluentAssetBuilder asset(String path);

  /**
   * List the assets under this component; returns an immutable collection.
   */
  Collection<FluentAsset> assets();

  /**
   * List the assets under this component; returns an immutable collection.
   */
  Collection<FluentAsset> assets(boolean useCache);

  /**
   * Update this component to have the given kind.
   *
   * @since 3.25
   */
  FluentComponent kind(String kind);

  /**
   * Deletes this component.
   */
  boolean delete();
}
