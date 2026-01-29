
package org.globalart.astrum.repository.config.internal;

import java.util.Collection;
import java.util.List;
import java.util.Set;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.datastore.ConfigStoreSupport;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.config.ConfigurationDAO;
import org.globalart.astrum.repository.config.ConfigurationStore;
import org.globalart.astrum.transaction.Transactional;

import com.google.common.collect.ImmutableList;
import org.apache.commons.collections4.CollectionUtils;

import static java.util.Collections.emptyList;
import static java.util.Optional.ofNullable;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * MyBatis {@link ConfigurationStore} implementation.
 *
 * @since 3.21
 */
@Component
@Qualifier("mybatis")
public class ConfigurationStoreImpl
    extends ConfigStoreSupport<ConfigurationDAO>
    implements ConfigurationStore
{
  @Autowired
  public ConfigurationStoreImpl(final DataSessionSupplier sessionSupplier) {
    super(sessionSupplier);
  }

  @Override
  public Configuration newConfiguration() {
    return new ConfigurationData();
  }

  @Transactional
  @Override
  public List<Configuration> list() {
    return ImmutableList.copyOf(dao().browse());
  }

  @Transactional
  @Override
  public void create(final Configuration configuration) {
    postCommitEvent(() -> new ConfigurationCreatedEvent((ConfigurationData) configuration));
    dao().create((ConfigurationData) configuration);

    dao().read(configuration.getRepositoryId())
        .ifPresent(persisted -> configuration.setAttributes(persisted.getAttributes()));
  }

  @Transactional
  @Override
  public void update(final Configuration configuration) {
    postCommitEvent(() -> new ConfigurationUpdatedEvent((ConfigurationData) configuration));
    dao().update((ConfigurationData) configuration);

    dao().read(configuration.getRepositoryId())
        .ifPresent(persisted -> configuration.setAttributes(persisted.getAttributes()));
  }

  @Transactional
  @Override
  public void delete(final Configuration configuration) {
    postCommitEvent(() -> new ConfigurationDeletedEvent((ConfigurationData) configuration));
    dao().deleteByName(configuration.getRepositoryName());
  }

  @Transactional
  @Override
  public Collection<Configuration> readByNames(final Set<String> repositoryNames) {
    return ofNullable(repositoryNames)
        .filter(CollectionUtils::isNotEmpty)
        .map(names -> dao().readByNames(names))
        .orElse(emptyList());
  }

  @Transactional
  @Override
  public boolean exists(final String repositoryName) {
    return dao().readByName(repositoryName).isPresent();
  }

  @Transactional
  @Override
  public Collection<Configuration> readByRecipe(final String recipeName) {
    return dao().readByRecipe(recipeName);
  }
}
