
package org.globalart.astrum.internal.datastore;

import java.util.Optional;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.datastore.api.DataStore;
import org.globalart.astrum.datastore.api.DataStoreManager;

import org.flywaydb.core.api.MigrationVersion;
import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;
import org.mockito.Mock;

import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.Silent.class)
public class DatabaseCheckImplTest
{
  private DatabaseCheckImpl underTest;

  @Mock
  private DataStoreManager dataStoreManager;

  private WithAnnotation withAnnotation;

  private WithoutAnnotation withoutAnnotation;

  private WithFromAnnotation withFromAnnotation;

  private WithLatestFromAnnotation withLatestFromAnnotation;

  @Mock
  private Optional<DataStore<?>> optionalDataStore;

  @Mock
  private DataStore dataStore;

  @Before
  public void setup() {
    when(dataStoreManager.get(any())).thenReturn(optionalDataStore);
    when(optionalDataStore.orElseThrow(any())).thenReturn(dataStore);

    withAnnotation = new WithAnnotation();
    withoutAnnotation = new WithoutAnnotation();
    withFromAnnotation = new WithFromAnnotation();
    withLatestFromAnnotation = new WithLatestFromAnnotation();
  }

  @Test
  public void testIsAllowedByVersion() {
    underTest = new DatabaseCheckImplForTest(dataStoreManager);

    assertTrue(underTest.isAllowedByVersion(withAnnotation.getClass()));
    assertFalse(underTest.isAllowedByVersion(withoutAnnotation.getClass()));
    assertTrue(underTest.isAllowedByVersion(withFromAnnotation.getClass()));
    assertFalse(underTest.isAllowedByVersion(withLatestFromAnnotation.getClass()));
  }

  private static class DatabaseCheckImplForTest
      extends DatabaseCheckImpl
  {
    public DatabaseCheckImplForTest(
        final DataStoreManager dataStoreManager)
    {
      super(dataStoreManager);
    }

    @Override
    MigrationVersion getMigrationVersion() {
      return MigrationVersion.fromVersion("1.4");
    }
  }

  private static class WithoutAnnotation
  {
  }

  @AvailabilityVersion(from = "1.0")
  private static class WithAnnotation
  {
  }

  @AvailabilityVersion(from = "1.3")
  private static class WithFromAnnotation
  {
  }

  @AvailabilityVersion(from = "1.6")
  private static class WithLatestFromAnnotation
  {
  }

}
