
package org.globalart.astrum.internal.capability.storage.datastore.cleanup;

import java.sql.Connection;
import java.sql.SQLException;
import java.util.Map;

import org.globalart.astrum.internal.capability.storage.CapabilityStorageItemDAO;
import org.globalart.astrum.internal.capability.storage.CapabilityStorageItemTestSupport;
import org.globalart.astrum.testdb.DataSessionConfiguration;
import org.globalart.astrum.testdb.DatabaseTest;
import org.globalart.astrum.testdb.TestDataSessionSupplier;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.junit.jupiter.api.Assertions.assertDoesNotThrow;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.mockito.Mockito.verify;

/**
 * Real-database tests for {@link CleanupCapabilityDuplicatesMigrationStep_1_27}: it delegates duplicate
 * cleanup to the service, then creates a unique (type, properties) constraint/index on
 * {@code capability_storage_item}.
 */
@ExtendWith(MockitoExtension.class)
class CleanupCapabilityDuplicatesMigrationStep_1_27Test
{
  @DataSessionConfiguration(daos = {CapabilityStorageItemDAO.class})
  TestDataSessionSupplier dataSessionSupplier;

  @Mock
  private CleanupCapabilityDuplicatesService service;

  private CleanupCapabilityDuplicatesMigrationStep_1_27 underTest;

  @BeforeEach
  void setUp() {
    underTest = new CleanupCapabilityDuplicatesMigrationStep_1_27(service);
  }

  @DatabaseTest
  void migrate_runsCleanupThenEnforcesUniqueness() throws Exception {
    try (Connection conn = dataSessionSupplier.openConnection()) {
      underTest.migrate(conn);

      verify(service).doCleanup();

      // First row of (type, properties) is accepted; an identical (type, properties) must now be rejected.
      CapabilityStorageItemTestSupport.insert(conn, "rapture.settings", Map.of("a", "1"));
      assertThrows(SQLException.class,
          () -> CapabilityStorageItemTestSupport.insert(conn, "rapture.settings", Map.of("a", "1")));
    }
  }

  @DatabaseTest
  void migrate_isIdempotent() throws Exception {
    try (Connection conn = dataSessionSupplier.openConnection()) {
      underTest.migrate(conn);
      assertDoesNotThrow(() -> underTest.migrate(conn));
    }
  }
}
