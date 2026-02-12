
package org.globalart.astrum.repository.content.store;

import org.junit.jupiter.api.BeforeEach;
import org.globalart.astrum.testdb.DatabaseTest;

/**
 * Test {@link ComponentDAO} with no entity versioning
 */
class UnversionedComponentDAOTest
    extends ComponentDAOTestSupport
{
  @BeforeEach
  void setup() {
    setupContent(false);
  }

  @Override
  @DatabaseTest
  protected void testCrudOperations() throws InterruptedException {
    super.testCrudOperations();
  }

  @Override
  @DatabaseTest
  protected void testBrowseComponentCoordinates() {
    super.testBrowseComponentCoordinates();
  }

  @Override
  @DatabaseTest
  protected void testContinuationBrowsing() {
    super.testContinuationBrowsing();
  }

  @Override
  @DatabaseTest
  protected void testDeleteAllComponents() {
    super.testDeleteAllComponents();
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
  protected void testBrowseComponentsInRepositories() {
    super.testBrowseComponentsInRepositories();
  }

  @Override
  @DatabaseTest
  protected void testFilterClauseIsolation() {
    super.testFilterClauseIsolation();
  }

  @Override
  @DatabaseTest
  protected void testContinuationSetBrowsing() {
    super.testContinuationSetBrowsing();
  }

  @Override
  @DatabaseTest
  protected void testNormalizationMethods() {
    super.testNormalizationMethods();
  }

  @Override
  @DatabaseTest
  protected void testBrowseComponentsEager() {
    super.testBrowseComponentsEager();
  }

  @Override
  @DatabaseTest
  protected void testBrowseVersionsByRepoIds() {
    super.testBrowseVersionsByRepoIds();
  }

  @Override
  @DatabaseTest
  protected void testReadCoordinateInRepoIds() {
    super.testReadCoordinateInRepoIds();
  }

  @Override
  @DatabaseTest(h2 = false, postgresql = true)
  protected void testCreateComponentReturnsNullIdWhenRepositoryDeleted() {
    super.testCreateComponentReturnsNullIdWhenRepositoryDeleted();
  }
}
