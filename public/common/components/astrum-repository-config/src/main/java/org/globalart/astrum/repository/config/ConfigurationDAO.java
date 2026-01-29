
package org.globalart.astrum.repository.config;

import java.util.Collection;
import java.util.Set;

import org.globalart.astrum.common.entity.EntityId;
import org.globalart.astrum.datastore.api.IterableDataAccess;
import org.globalart.astrum.repository.config.internal.ConfigurationData;

/**
 * {@link ConfigurationData} access.
 *
 * @since 3.21
 */
public interface ConfigurationDAO
    extends IterableDataAccess.WithName<ConfigurationData>
{
  Collection<Configuration> readByNames(Set<String> repositoryNames);

  Collection<ConfigurationData> readByIds(Collection<EntityId> repositoryIds);

  Collection<Configuration> readByRecipe(String recipeName);
}
