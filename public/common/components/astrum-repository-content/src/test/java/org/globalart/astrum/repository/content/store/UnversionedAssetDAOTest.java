
package org.globalart.astrum.repository.content.store;

import org.junit.jupiter.api.BeforeEach;
import org.globalart.astrum.testdb.DatabaseTest;

/**
 * Test {@link AssetDAO} without setting the component's entity version
 */
class UnversionedAssetDAOTest
    extends AssetDAOTestSupport
{
  @BeforeEach
  void setupContent() {
    initialiseContent(false);
  }

  @Override
  @DatabaseTest
  protected void testCrudOperations() throws InterruptedException {
    super.testCrudOperations();
  }

  @Override
  @DatabaseTest
  protected void testLastDownloaded() throws InterruptedException {
    super.testLastDownloaded();
  }

  @Override
  @DatabaseTest
  protected void testAttachingBlobs() throws InterruptedException {
    super.testAttachingBlobs();
  }

  @Override
  @DatabaseTest
  protected void testBrowseComponentAssets() {
    super.testBrowseComponentAssets();
  }

  @Override
  @DatabaseTest
  protected void testContinuationBrowsing() {
    super.testContinuationBrowsing();
  }

  @Override
  @DatabaseTest
  protected void testFlaggedBrowsing() {
    super.testFlaggedBrowsing();
  }

  @Override
  @DatabaseTest
  protected void testReadPathTest() {
    super.testReadPathTest();
  }

  @Override
  @DatabaseTest
  protected void testDeleteAllAssets() {
    super.testDeleteAllAssets();
  }

  @Override
  @DatabaseTest
  protected void testReadPaths() {
    super.testReadPaths();
  }

  @Override
  @DatabaseTest
  protected void testPurgeOperation() {
    super.testPurgeOperation();
  }

  @Override
  @DatabaseTest
  protected void testRoundTrip() {
    super.testRoundTrip();
  }

  @Override
  @DatabaseTest
  protected void testBrowseAssetsInRepositories() {
    super.testBrowseAssetsInRepositories();
  }

  @Override
  @DatabaseTest
  protected void testBrowseEagerAssetsInRepository() {
    super.testBrowseEagerAssetsInRepository();
  }

  @Override
  @DatabaseTest
  protected void testBrowseEagerAssetsInRepositoryOrderingByBlobCreated() throws InterruptedException {
    super.testBrowseEagerAssetsInRepositoryOrderingByBlobCreated();
  }

  @Override
  @DatabaseTest
  protected void testBrowseEagerAssetsInRepositoryPaginationWithBlobCreated() throws InterruptedException {
    super.testBrowseEagerAssetsInRepositoryPaginationWithBlobCreated();
  }

  @Override
  @DatabaseTest
  protected void testBrowseEagerAssetsInRepositorySameTimestampDifferentAssetId() throws InterruptedException {
    super.testBrowseEagerAssetsInRepositorySameTimestampDifferentAssetId();
  }

  @Override
  @DatabaseTest
  protected void testSetLastDownloaded() {
    super.testSetLastDownloaded();
  }

  @Override
  @DatabaseTest
  protected void testLastUpdated() {
    super.testLastUpdated();
  }

  @Override
  @DatabaseTest
  protected void testFilterClauseIsolation() {
    super.testFilterClauseIsolation();
  }

  @Override
  @DatabaseTest
  protected void testFindByBlobRef() throws InterruptedException {
    super.testFindByBlobRef();
  }

  @Override
  @DatabaseTest
  protected void testFindByComponentIds() {
    super.testFindByComponentIds();
  }

  @Override
  @DatabaseTest
  protected void testFindAddedToRepository() {
    super.testFindAddedToRepository();
  }

  @Override
  @DatabaseTest
  protected void testFindAddedToRepositoryTruncatesToMilliseconds() {
    super.testFindAddedToRepositoryTruncatesToMilliseconds();
  }

  @Override
  @DatabaseTest
  protected void testDeleteByPaths() {
    super.testDeleteByPaths();
  }

  @Override
  @DatabaseTest
  protected void testAssetRecordsExist() {
    super.testAssetRecordsExist();
  }

  @Override
  @DatabaseTest
  protected void testMarkAsDownloadedNeverDownloadedUpdatesRow() {
    super.testMarkAsDownloadedNeverDownloadedUpdatesRow();
  }

  @Override
  @DatabaseTest
  protected void testMarkAsDownloadedRecentlyDownloadedSkipsUpdate() {
    super.testMarkAsDownloadedRecentlyDownloadedSkipsUpdate();
  }

  @Override
  @DatabaseTest
  protected void testMarkAsDownloadedZeroIntervalAlwaysUpdates() {
    super.testMarkAsDownloadedZeroIntervalAlwaysUpdates();
  }
}
