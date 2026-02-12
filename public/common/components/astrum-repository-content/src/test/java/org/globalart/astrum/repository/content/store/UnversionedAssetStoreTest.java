
package org.globalart.astrum.repository.content.store;

import org.junit.jupiter.api.BeforeEach;
import org.globalart.astrum.testdb.DatabaseTest;

class UnversionedAssetStoreTest
    extends AssetStoreTestSupport
{
  @BeforeEach
  void setup() {
    initialiseStores(false);
  }

  @Override
  @DatabaseTest
  protected void testDeleteAsset() {
    super.testDeleteAsset();
  }

  @Override
  @DatabaseTest
  protected void testBrowseUpdatedAssetsDifferentDates() {
    super.testBrowseUpdatedAssetsDifferentDates();
  }

  @Override
  @DatabaseTest
  protected void testBrowseUpdatedAssetsIdenticalDates() {
    super.testBrowseUpdatedAssetsIdenticalDates();
  }

  @Override
  @DatabaseTest
  protected void testDeleteAssetsByPaths() {
    super.testDeleteAssetsByPaths();
  }

  @Override
  @DatabaseTest
  protected void testMarkAsDownloadedReturnsTrueWhenRowUpdated() {
    super.testMarkAsDownloadedReturnsTrueWhenRowUpdated();
  }

  @Override
  @DatabaseTest
  protected void testMarkAsDownloadedReturnsFalseWhenThrottled() {
    super.testMarkAsDownloadedReturnsFalseWhenThrottled();
  }

  @Override
  @DatabaseTest
  protected void testConcurrentMarkAsDownloadedOnlyOneSucceeds() throws Exception {
    super.testConcurrentMarkAsDownloadedOnlyOneSucceeds();
  }
}
