
package org.globalart.astrum.security.realm;

import javax.annotation.Nullable;

/**
 * {@link RealmConfiguration} store.
 *
 * @since 3.0
 */
public interface RealmConfigurationStore
{
  /**
   * @since 3.20
   */
  RealmConfiguration newEntity();

  @Nullable
  RealmConfiguration load();

  void save(RealmConfiguration configuration);
}
