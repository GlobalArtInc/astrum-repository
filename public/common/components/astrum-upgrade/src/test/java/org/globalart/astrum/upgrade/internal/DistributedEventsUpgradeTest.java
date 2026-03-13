
package org.globalart.astrum.upgrade.internal;

import java.sql.Connection;

import org.globalart.astrum.testdb.DataSessionConfiguration;
import org.globalart.astrum.testdb.TestDataSessionSupplier;

import org.globalart.astrum.testdb.DatabaseTest;

import static org.mockito.Mockito.spy;
import static org.mockito.Mockito.verifyNoInteractions;

class DistributedEventsUpgradeTest
{
  @DataSessionConfiguration(daos = {})
  TestDataSessionSupplier supplier;

  @DatabaseTest
  void shouldBeNoOp() throws Exception {
    DistributedEventsUpgrade underTest = new DistributedEventsUpgrade();
    try (Connection conn = supplier.openConnection()) {
      Connection spyConn = spy(conn);
      underTest.migrate(spyConn);
      verifyNoInteractions(spyConn);
    }
  }
}
