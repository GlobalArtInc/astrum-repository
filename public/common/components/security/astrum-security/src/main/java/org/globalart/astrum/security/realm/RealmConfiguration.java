
package org.globalart.astrum.security.realm;

import java.util.List;

import javax.annotation.Nullable;

/**
 * RealmConfiguration interface
 *
 * @since 3.0
 */
public interface RealmConfiguration
{
  List<String> getRealmNames();

  void setRealmNames(@Nullable List<String> realmNames);

  RealmConfiguration copy();
}
