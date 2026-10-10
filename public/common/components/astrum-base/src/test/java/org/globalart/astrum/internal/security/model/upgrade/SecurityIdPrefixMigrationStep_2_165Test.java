package org.globalart.astrum.internal.security.model.upgrade;

import java.sql.Connection;
import java.util.Optional;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verifyNoInteractions;

public class SecurityIdPrefixMigrationStep_2_165Test
{
  private final SecurityIdPrefixMigrationStep_2_165 underTest = new SecurityIdPrefixMigrationStep_2_165();

  @Test
  public void testVersion() {
    assertThat(underTest.version(), is(Optional.of("2.165")));
  }

  @Test
  public void testMigrateLeavesTheDatabaseAlone() {
    Connection connection = mock(Connection.class);

    underTest.migrate(connection);

    verifyNoInteractions(connection);
  }
}
