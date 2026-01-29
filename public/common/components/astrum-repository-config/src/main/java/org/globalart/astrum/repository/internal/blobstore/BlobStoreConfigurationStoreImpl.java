
package org.globalart.astrum.repository.internal.blobstore;

import java.util.List;
import java.util.Optional;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;
import org.globalart.astrum.datastore.ConfigStoreSupport;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.repository.blobstore.BlobStoreConfigurationStore;
import org.globalart.astrum.transaction.Transactional;

import com.google.common.collect.ImmutableList;

import static org.globalart.astrum.blobstore.group.BlobStoreGroupConfigurationHelper.memberNames;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * MyBatis {@link BlobStoreConfigurationStore} implementation.
 *
 * @since 3.21
 */
@Component
@Qualifier("mybatis")
public class BlobStoreConfigurationStoreImpl
    extends ConfigStoreSupport<BlobStoreConfigurationDAO>
    implements BlobStoreConfigurationStore
{
  @Autowired
  public BlobStoreConfigurationStoreImpl(final DataSessionSupplier sessionSupplier) {
    super(sessionSupplier);
  }

  @Override
  public BlobStoreConfiguration newConfiguration() {
    return new BlobStoreConfigurationData();
  }

  @Transactional
  @Override
  public List<BlobStoreConfiguration> list() {
    return ImmutableList.copyOf(dao().browse());
  }

  @Transactional
  @Override
  public void create(final BlobStoreConfiguration configuration) {
    dao().create((BlobStoreConfigurationData) configuration);
  }

  @Transactional
  @Override
  public BlobStoreConfiguration read(final String name) {
    return dao().readByName(name).orElse(null);
  }

  @Transactional
  @Override
  public void update(final BlobStoreConfiguration configuration) {
    dao().update((BlobStoreConfigurationData) configuration);
  }

  @Transactional
  @Override
  public void delete(final BlobStoreConfiguration configuration) {
    dao().deleteByName(configuration.getName());
  }

  @Transactional
  @Override
  public Optional<BlobStoreConfiguration> findParent(final String name) {
    return dao().findCandidateParents(name)
        .stream()
        .filter(config -> memberNames(config).contains(name))
        .findFirst();
  }
}
