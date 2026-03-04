
package org.globalart.astrum.selector;

import java.util.List;

import org.globalart.astrum.common.lifecycle.Lifecycle;
import org.globalart.astrum.common.entity.EntityId;

/**
 * {@link SelectorConfiguration} store.
 *
 * since 3.0
 */
public interface SelectorConfigurationStore
    extends Lifecycle
{
  /**
   * @return all configuration
   */
  List<SelectorConfiguration> browse();

  /**
   * @return configuration by id
   */
  SelectorConfiguration read(EntityId entityId);

  /**
   * @since 3.6
   * @return configuration by name
   */
  SelectorConfiguration getByName(String name);

  /**
   * Persist a new configuration.
   */
  void create(SelectorConfiguration configuration);

  /**
   * Persist an existing configuration.
   */
  void update(SelectorConfiguration configuration);

  /**
   * Delete an existing configuration.
   */
  void delete(SelectorConfiguration configuration);

  SelectorConfiguration newSelectorConfiguration();
}
