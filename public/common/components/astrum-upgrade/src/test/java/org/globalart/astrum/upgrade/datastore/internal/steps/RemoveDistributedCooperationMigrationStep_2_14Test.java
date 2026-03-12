
package org.globalart.astrum.upgrade.datastore.internal.steps;

import java.sql.Connection;

import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;

import static org.mockito.Mockito.verifyNoInteractions;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class RemoveDistributedCooperationMigrationStep_2_14Test
{
  @Mock
  private Connection connection;

  private RemoveDistributedCooperationMigrationStep_2_14 underTest;

  @Before
  public void setup() {
    underTest =
        new RemoveDistributedCooperationMigrationStep_2_14();
  }

  @Test
  public void testNoAction() throws Exception {
    // as this upgrade step has become a noop, just validate that nothing actually happens
    underTest.migrate(connection);
    verifyNoInteractions(connection);
  }
}
