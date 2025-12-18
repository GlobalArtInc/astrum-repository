
package org.globalart.astrum.security.internal;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.security.realm.RealmConfiguration;
import org.globalart.astrum.security.realm.RealmConfigurationStore;

import com.google.common.collect.Lists;
import org.springframework.beans.factory.FactoryBean;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Initial {@link RealmConfiguration} provider.
 *
 * @since 3.0
 */
@Component
@Qualifier("initial")
public class InitialRealmConfigurationProvider
    implements FactoryBean<RealmConfiguration>
{
  private final RealmConfigurationStore store;

  @Autowired
  public InitialRealmConfigurationProvider(final RealmConfigurationStore store) {
    this.store = checkNotNull(store);
  }

  @Override
  public RealmConfiguration getObject() {
    RealmConfiguration configuration = store.newEntity();
    configuration.setRealmNames(Lists.newArrayList(
        AuthenticatingRealmImpl.NAME,
        AuthorizingRealmImpl.NAME));
    return configuration;
  }

  @Override
  public Class<?> getObjectType() {
    return RealmConfiguration.class;
  }
}
