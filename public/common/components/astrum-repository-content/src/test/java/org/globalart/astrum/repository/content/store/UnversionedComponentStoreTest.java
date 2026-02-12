
package org.globalart.astrum.repository.content.store;

import org.junit.jupiter.api.BeforeEach;
import org.globalart.astrum.testdb.DatabaseTest;

class UnversionedComponentStoreTest
    extends ComponentStoreTestSupport
{
  @BeforeEach
  void setup() {
    initialiseStores(false);
  }

  @Override
  @DatabaseTest
  protected void testPurge_byComponentIds() {
    super.testPurge_byComponentIds();
  }

  @Override
  @DatabaseTest
  protected void testPurge_byComponent() {
    super.testPurge_byComponent();
  }

  @Override
  @DatabaseTest
  protected void testPurge_byComponent_resolvesAssetComponents() throws Exception {
    super.testPurge_byComponent_resolvesAssetComponents();
  }

  @Override
  @DatabaseTest
  protected void testPurge_byComponent_browsesEveryAssetPage() {
    super.testPurge_byComponent_browsesEveryAssetPage();
  }
}
