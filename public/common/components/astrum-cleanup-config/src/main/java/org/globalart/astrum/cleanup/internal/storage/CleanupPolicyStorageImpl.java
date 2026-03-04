
package org.globalart.astrum.cleanup.internal.storage;

import java.util.List;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.cleanup.storage.CleanupPolicy;
import org.globalart.astrum.cleanup.storage.CleanupPolicyStorage;
import org.globalart.astrum.cleanup.storage.event.CleanupPolicyDeletedEvent;
import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.datastore.ConfigStoreSupport;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.transaction.Transactional;

import com.google.common.collect.ImmutableList;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * MyBatis {@link CleanupPolicyStorage} implementation.
 *
 * @since 3.21
 */
@Component
@Qualifier("mybatis")
public class CleanupPolicyStorageImpl
    extends ConfigStoreSupport<CleanupPolicyDAO>
    implements CleanupPolicyStorage
{
  private final EventManager eventManager;

  @Autowired
  public CleanupPolicyStorageImpl(final DataSessionSupplier sessionSupplier, final EventManager eventManager) {
    super(sessionSupplier);
    this.eventManager = checkNotNull(eventManager);
  }

  @Override
  public CleanupPolicy newCleanupPolicy() {
    return new CleanupPolicyData();
  }

  @Transactional
  @Override
  public List<CleanupPolicy> getAll() {
    return ImmutableList.copyOf(dao().browse());
  }

  @Transactional
  @Override
  public List<CleanupPolicy> getAllByFormat(final String format) {
    return ImmutableList.copyOf(dao().browseByFormat(format));
  }

  @Transactional
  @Override
  public long count() {
    return dao().count();
  }

  @Transactional
  @Override
  public CleanupPolicy add(final CleanupPolicy policy) {
    dao().create((CleanupPolicyData) policy);
    return policy;
  }

  @Transactional
  @Override
  public CleanupPolicy get(final String policyName) {
    return dao().read(policyName).orElse(null);
  }

  @Transactional
  @Override
  public boolean exists(final String policyName) {
    return dao().read(policyName).isPresent();
  }

  @Transactional
  @Override
  public CleanupPolicy update(final CleanupPolicy policy) {
    dao().update((CleanupPolicyData) policy);
    return policy;
  }

  @Override
  public void remove(final CleanupPolicy policy) {
    doRemove(policy);
    postDeletedEvent(policy);
  }

  @Transactional
  protected void doRemove(final CleanupPolicy policy) {
    dao().delete(policy.getName());
  }

  private void postDeletedEvent(final CleanupPolicy policy) {
    // notify cleanup policy has been removed
    eventManager.post(new CleanupPolicyDeletedEvent()
    {
      @Override
      public boolean isLocal() {
        return true;
      }

      @Override
      public CleanupPolicy getCleanupPolicy() {
        return policy;
      }
    });
  }
}
