
package org.globalart.astrum.scheduling.internal.upgrade.datastore;

import java.time.OffsetDateTime;
import java.util.Collection;
import java.util.Map;

import org.globalart.astrum.testdb.DataSessionConfiguration;

import org.globalart.astrum.testdb.TestDataSessionSupplier;

import com.google.common.collect.Iterables;
import org.globalart.astrum.testdb.DatabaseTest;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.Matchers.hasSize;
import static org.hamcrest.Matchers.is;

class UpgradeTaskDAOTest

{
  @DataSessionConfiguration(daos = UpgradeTaskDAO.class)
  TestDataSessionSupplier dataSession;

  @DatabaseTest
  void testBrowse() {
    dataSession.callDAO(UpgradeTaskDAO.class, dao -> {
      dao.create(new UpgradeTaskData("task-one", Map.of()));
      dao.create(new UpgradeTaskData("task-two", Map.of()));
    });

    Collection<UpgradeTaskData> tasks =
        (Collection<UpgradeTaskData>) dataSession.withDAO(UpgradeTaskDAO.class, dao -> dao.browse());

    assertThat(tasks, hasSize(2));
    assertThat(tasks.stream().map(UpgradeTaskData::getTaskId).toList(), contains("task-one", "task-two"));
  }

  @DatabaseTest
  void testBrowseBefore() {
    dataSession.callDAO(UpgradeTaskDAO.class, dao -> dao.create(new UpgradeTaskData("task-one", Map.of())));

    OffsetDateTime before = OffsetDateTime.now();

    dataSession.callDAO(UpgradeTaskDAO.class, dao -> dao.create(new UpgradeTaskData("task-two", Map.of())));

    Collection<UpgradeTaskData> tasks =
        (Collection<UpgradeTaskData>) dataSession.withDAO(UpgradeTaskDAO.class, dao -> dao.browseBefore(before));

    assertThat(tasks, hasSize(1));
    assertThat(Iterables.getOnlyElement(tasks).getTaskId(), is("task-one"));
  }
}
