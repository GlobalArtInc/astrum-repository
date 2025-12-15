
package org.globalart.astrum.internal.capability.storage.datastore.cleanup;

import java.util.HashMap;
import java.util.Map;

import org.globalart.astrum.internal.capability.storage.CapabilityStorageItemDAO;
import org.globalart.astrum.internal.capability.storage.CapabilityStorageItemTestSupport;
import org.globalart.astrum.internal.capability.storage.upgrade.UpgradeCapabilityStorage;
import org.globalart.astrum.testdb.DataSessionConfiguration;
import org.globalart.astrum.testdb.DatabaseTest;
import org.globalart.astrum.testdb.TestDataSessionSupplier;

import static java.util.Collections.emptyMap;
import static org.assertj.core.api.Assertions.assertThat;

/**
 * Real-database tests for {@link CleanupCapabilityDuplicatesService}, exercising the
 * {@link UpgradeCapabilityStorage} direct-SQL path on {@code capability_storage_item}.
 */
class CleanupCapabilityDuplicatesServiceTest
{
  @DataSessionConfiguration(daos = {CapabilityStorageItemDAO.class})
  TestDataSessionSupplier dataSessionSupplier;

  private UpgradeCapabilityStorage store() {
    return new UpgradeCapabilityStorage(dataSessionSupplier);
  }

  private CleanupCapabilityDuplicatesService service() {
    return new CleanupCapabilityDuplicatesService(store());
  }

  @DatabaseTest
  void cleanup_removesDuplicatesKeepingOnePerGroup() throws Exception {
    insertN(5, "test-capability-1", emptyMap());
    insertN(5, "test-capability-2", props("repository", "maven-central"));
    insertN(1, "test-capability-2", props("repository", "maven-proxy"));
    insertN(5, "test-capability-3", props("repository", "nuget-proxy"));
    insertN(5, "test-capability-3", props("repository", "nuget-group"));
    insertN(2, "test-capability-4", props("repository", "nuget-group", "auth", "false"));

    CleanupCapabilityDuplicatesService underTest = service();
    assertThat(store().getAll()).hasSize(23);
    assertThat(underTest.browseCapabilityDuplicates().keySet()).hasSize(5);

    underTest.doCleanup();

    assertThat(store().getAll()).hasSize(6);
    assertThat(underTest.browseCapabilityDuplicates()).isEmpty();
  }

  @DatabaseTest
  void cleanup_doesNotTouchUniqueRecords() throws Exception {
    insertN(1, "test-capability-1", emptyMap());
    insertN(1, "test-capability-1", props("repository", "maven-central"));
    insertN(1, "test-capability-2", emptyMap());
    insertN(1, "test-capability-2", props("repository", "maven-central", "test", "test"));
    insertN(1, "test-capability-3", emptyMap());

    CleanupCapabilityDuplicatesService underTest = service();
    assertThat(underTest.browseCapabilityDuplicates()).isEmpty();

    underTest.doCleanup();

    assertThat(store().getAll()).hasSize(5);
  }

  @DatabaseTest
  void cleanup_notNeeded_isNoOp() {
    CleanupCapabilityDuplicatesService underTest = service();

    underTest.doCleanup();

    assertThat(underTest.browseCapabilityDuplicates()).isEmpty();
  }

  private static Map<String, String> props(final String... kv) {
    Map<String, String> map = new HashMap<>();
    for (int i = 0; i < kv.length; i += 2) {
      map.put(kv[i], kv[i + 1]);
    }
    return map;
  }

  private void insertN(final int count, final String type, final Map<String, String> properties) throws Exception {
    for (int i = 0; i < count; i++) {
      CapabilityStorageItemTestSupport.insert(dataSessionSupplier, type, properties);
    }
  }
}
