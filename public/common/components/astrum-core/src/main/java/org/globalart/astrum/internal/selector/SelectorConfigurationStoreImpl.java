
package org.globalart.astrum.internal.selector;

import java.util.List;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.entity.EntityId;
import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.datastore.ConfigStoreSupport;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.distributed.event.service.api.EventType;
import org.globalart.astrum.distributed.event.service.api.common.SelectorConfigurationChangedEvent;
import org.globalart.astrum.selector.SelectorConfiguration;
import org.globalart.astrum.selector.SelectorConfigurationStore;
import org.globalart.astrum.transaction.Transactional;

import com.google.common.collect.ImmutableList;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * MyBatis {@link SelectorConfigurationStore} implementation.
 *
 * @since 3.21
 */
@Component
@Qualifier("mybatis")
public class SelectorConfigurationStoreImpl
    extends ConfigStoreSupport<SelectorConfigurationDAO>
    implements SelectorConfigurationStore
{
  private final EventManager eventManager;

  @Autowired
  public SelectorConfigurationStoreImpl(final DataSessionSupplier sessionSupplier, final EventManager eventManager) {
    super(sessionSupplier);
    this.eventManager = checkNotNull(eventManager);
  }

  @Override
  public SelectorConfiguration newSelectorConfiguration() {
    return new SelectorConfigurationData();
  }

  @Transactional
  @Override
  public List<SelectorConfiguration> browse() {
    return ImmutableList.copyOf(dao().browse());
  }

  @Override
  public void create(final SelectorConfiguration configuration) {
    doCreate(configuration);
    postEvent(configuration);
    postDesEvent(EventType.CREATED);
  }

  @Transactional
  protected void doCreate(final SelectorConfiguration configuration) {
    dao().create((SelectorConfigurationData) configuration);
  }

  @Override
  public SelectorConfiguration read(final EntityId entityId) {
    throw new UnsupportedOperationException("Use getByName instead");
  }

  @Override
  public void update(final SelectorConfiguration configuration) {
    if (doUpdate(configuration)) {
      postEvent(configuration);
      postDesEvent(EventType.UPDATED);
    }
  }

  @Transactional
  protected boolean doUpdate(final SelectorConfiguration configuration) {
    return dao().update((SelectorConfigurationData) configuration);
  }

  @Override
  public void delete(final SelectorConfiguration configuration) {
    if (doDelete(configuration)) {
      postEvent(configuration);
      postDesEvent(EventType.DELETED);
    }
  }

  @Transactional
  protected boolean doDelete(final SelectorConfiguration configuration) {
    return dao().delete(configuration.getName());
  }

  @Transactional
  @Override
  public SelectorConfiguration getByName(final String name) {
    return dao().read(name).orElse(null);
  }

  private void postEvent(final SelectorConfiguration configuration) {
    // trigger invalidation of SelectorManagerImpl caches
    eventManager.post(new SelectorConfigurationEvent()
    {
      @Override
      public boolean isLocal() {
        return true;
      }

      @Override
      public SelectorConfiguration getSelectorConfiguration() {
        return configuration;
      }
    });
  }

  private void postDesEvent(final EventType eventType) {
    eventManager.post(new SelectorConfigurationChangedEvent(eventType));
  }
}
