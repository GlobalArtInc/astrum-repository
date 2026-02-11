
package org.globalart.astrum.repository.content.browse.capability;

import org.globalart.astrum.common.db.DatabaseCheck;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.Silent.class)
public class BrowseTrimServiceTest
{
  @Mock
  private DatabaseCheck databaseCheck;

  private BrowseTrimService underTest;

  @Before
  public void setup() {
    underTest = new BrowseTrimService(databaseCheck);
  }

  @Test
  public void testShouldAllowTrim_postgresqlWithCapabilityDisabled() {
    when(databaseCheck.isPostgresql()).thenReturn(true);
    underTest.setPostgresqlTrimEnabled(false);

    boolean result = underTest.shouldAllowTrim();

    assertFalse("PostgreSQL should not allow trim when capability is disabled", result);
  }

  @Test
  public void testShouldAllowTrim_postgresqlWithCapabilityEnabled() {
    when(databaseCheck.isPostgresql()).thenReturn(true);
    underTest.setPostgresqlTrimEnabled(true);

    boolean result = underTest.shouldAllowTrim();

    assertTrue("PostgreSQL should allow trim when capability is enabled", result);
  }

  @Test
  public void testShouldAllowTrim_h2AlwaysAllows() {
    when(databaseCheck.isPostgresql()).thenReturn(false);
    underTest.setPostgresqlTrimEnabled(false);

    boolean result = underTest.shouldAllowTrim();

    assertTrue("H2 should always allow trim regardless of capability setting", result);
  }

  @Test
  public void testShouldAllowTrim_h2AlwaysAllowsEvenWithCapabilityEnabled() {
    when(databaseCheck.isPostgresql()).thenReturn(false);
    underTest.setPostgresqlTrimEnabled(true);

    boolean result = underTest.shouldAllowTrim();

    assertTrue("H2 should allow trim", result);
  }

  @Test
  public void testPostgresqlTrimEnabled_defaultsToFalse() {
    assertFalse("PostgreSQL trim should be disabled by default", underTest.isPostgresqlTrimEnabled());
  }

  @Test
  public void testPostgresqlTrimEnabled_toggles() {
    assertFalse(underTest.isPostgresqlTrimEnabled());

    underTest.setPostgresqlTrimEnabled(true);
    assertTrue(underTest.isPostgresqlTrimEnabled());

    underTest.setPostgresqlTrimEnabled(false);
    assertFalse(underTest.isPostgresqlTrimEnabled());
  }

  @Test
  public void testBatchTrimEnabled_defaultsToFalse() {
    assertFalse("Batch trim should be disabled by default", underTest.isBatchTrimEnabled());
  }

  @Test
  public void testBatchTrimEnabled_toggles() {
    assertFalse(underTest.isBatchTrimEnabled());

    underTest.setBatchTrimEnabled(true);
    assertTrue(underTest.isBatchTrimEnabled());

    underTest.setBatchTrimEnabled(false);
    assertFalse(underTest.isBatchTrimEnabled());
  }

  @Test
  public void testBothSettings_independent() {
    assertFalse(underTest.isPostgresqlTrimEnabled());
    assertFalse(underTest.isBatchTrimEnabled());

    underTest.setPostgresqlTrimEnabled(true);
    assertTrue(underTest.isPostgresqlTrimEnabled());
    assertFalse("Batch trim should remain independent", underTest.isBatchTrimEnabled());

    underTest.setBatchTrimEnabled(true);
    assertTrue(underTest.isPostgresqlTrimEnabled());
    assertTrue(underTest.isBatchTrimEnabled());
  }
}
