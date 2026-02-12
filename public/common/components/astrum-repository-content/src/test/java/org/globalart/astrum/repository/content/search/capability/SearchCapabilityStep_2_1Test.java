
package org.globalart.astrum.repository.content.search.capability;

import java.sql.Connection;
import java.util.Optional;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertTrue;

@RunWith(MockitoJUnitRunner.Silent.class)
public class SearchCapabilityStep_2_1Test
{
  @Mock
  private Connection connection;

  private SearchCapabilityStep_2_1 underTest;

  @Before
  public void setUp() {
    underTest = new SearchCapabilityStep_2_1();
  }

  @Test
  public void testVersion() {
    Optional<String> version = underTest.version();
    assertTrue(version.isPresent());
    assertEquals("2.1", version.get());
  }

  @Test
  public void testMigrate_isNoOp() throws Exception {
    // migrate is a no-op; verify it does not throw
    underTest.migrate(connection);
  }
}
