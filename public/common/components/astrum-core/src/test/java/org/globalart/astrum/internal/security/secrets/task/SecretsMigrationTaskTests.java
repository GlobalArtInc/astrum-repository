
package org.globalart.astrum.internal.security.secrets.task;

import java.util.ArrayList;
import java.util.List;

import org.globalart.astrum.security.secrets.SecretsMigrator;

import org.junit.Test;

import static org.junit.jupiter.api.Assertions.fail;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verify;

public class SecretsMigrationTaskTests
{
  @Test
  public void testTaskMigratesSeveralSources() throws Exception {
    List<SecretsMigrator> migrators = getMigrators();
    SecretsMigrationTask underTest = new SecretsMigrationTask(migrators);

    try {
      underTest.execute();
    }
    catch (Exception e) {
      fail(e.getMessage());
    }

    for (SecretsMigrator migrator : migrators) {
      verify(migrator).migrate();
    }
  }

  private List<SecretsMigrator> getMigrators() {
    List<SecretsMigrator> sources = new ArrayList<>();

    for (int i = 0; i < 5; i++) {
      SecretsMigrator migrationSource = mock(SecretsMigrator.class);
      sources.add(migrationSource);
    }

    return sources;
  }
}
