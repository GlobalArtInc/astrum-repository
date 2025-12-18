
package org.globalart.astrum.security.realm;

import java.util.List;

import org.globalart.astrum.common.lifecycle.Lifecycle;

/**
 * Realm manager.
 *
 * @since 3.0
 */
public interface RealmManager
    extends Lifecycle
{
  /**
   * Check if given realm-name is enabled.
   */
  boolean isRealmEnabled(String realmName);

  /**
   * Helper to enable or disable given realm-name.
   *
   * @see #enableRealm
   * @see #disableRealm
   */
  void enableRealm(String realmName, boolean enable);

  /**
   * Enable given realm-name, if not already enabled.
   */
  void enableRealm(String realmName);

  /**
   * Enable given realm-name at the desired zero-based index in realmList, if not already enabled.
   * If the index isn't valid will simply be inserted at end of list.
   */
  void enableRealm(final String realmName, final int index);

  /**
   * Disable given realm-name, if not already disabled.
   */
  void disableRealm(String realmName);

  /**
   * Get the list of known available realms.
   *
   * @since 3.20
   */
  List<SecurityRealm> getAvailableRealms();

  /**
   * Get the list of available realms
   *
   * @param includeHidden if true, include any "hidden" realms
   * @return
   */
  List<SecurityRealm> getAvailableRealms(final boolean includeHidden);

  /**
   * Get the list of configured realm Ids
   */
  List<String> getConfiguredRealmIds();

  /**
   * Get the list of configured realm Ids
   * 
   * @param includeHidden if true, include any "hidden" realms
   */
  List<String> getConfiguredRealmIds(final boolean includeHidden);

  /**
   * Set the list of configured realm Ids*
   */
  void setConfiguredRealmIds(final List<String> realmIds);
}
