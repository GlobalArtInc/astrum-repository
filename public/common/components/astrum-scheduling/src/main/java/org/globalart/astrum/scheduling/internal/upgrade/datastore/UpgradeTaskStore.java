
package org.globalart.astrum.scheduling.internal.upgrade.datastore;

import java.time.OffsetDateTime;
import java.util.Optional;
import java.util.stream.Stream;
import java.util.stream.StreamSupport;

import org.globalart.astrum.datastore.ConfigStoreSupport;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.transaction.Transactional;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
@Qualifier("mybatis")
public class UpgradeTaskStore
    extends ConfigStoreSupport<UpgradeTaskDAO>
{
  @Autowired
  public UpgradeTaskStore(final DataSessionSupplier sessionSupplier) {
    super(sessionSupplier, UpgradeTaskDAO.class);
  }

  @Transactional
  public void insert(final UpgradeTaskData task) {
    dao().create(task);
  }

  @Transactional
  public Optional<UpgradeTaskData> read(final int id) {
    return dao().read(id);
  }

  @Transactional
  public int markFailed(final String id) {
    return dao().setStatus(id, "failed");
  }

  @Transactional
  public int markCanceled(final String taskId) {
    return dao().setStatus(taskId, "canceled");
  }

  @Transactional
  public int deleteByTaskId(final String id) {
    return dao().deleteByTaskId(id);
  }

  @Transactional
  public Stream<UpgradeTaskData> browse() {
    return StreamSupport.stream(dao().browse().spliterator(), false);
  }

  @Transactional
  public void update(final UpgradeTaskData task) {
    dao().update(task);
  }

  @Transactional
  public Optional<UpgradeTaskData> next() {
    return dao().next();
  }

  /**
   * @param before the date tasks should have been created before
   * @return tasks scheduled by upgrades before the specified time
   */
  @Transactional
  public Iterable<UpgradeTaskData> browse(final OffsetDateTime before) {
    return dao().browseBefore(before);
  }
}
