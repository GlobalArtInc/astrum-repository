
package org.globalart.astrum.upgrade.datastore.internal;

import java.sql.Connection;
import java.util.Optional;

import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;

import org.junit.Test;

import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;
import static org.globalart.astrum.upgrade.datastore.internal.AstrumJavaMigration.nameMatcher;

public class AstrumJavaMigrationTest
{

  @Test
  public void testNameMatcher() {
    assertTrue(nameMatcher(AStep.class).test(toDescription(new AStep(), null)));
    assertTrue(nameMatcher(AStep.class).test(toDescription(new AStep(), 1)));

    assertFalse(nameMatcher(AStepA.class).test(toDescription(new AStep(), null)));
  }

  private static String toDescription(final DatabaseMigrationStep step, final Integer round) {
    return new AstrumJavaMigration(step, round).getDescription();
  }

  private static class AStep
      extends TestStep
  {
  }

  private static class AStepA
      extends TestStep
  {
  }

  private static class TestStep
      implements DatabaseMigrationStep
  {

    @Override
    public Optional<String> version() {
      return Optional.empty();
    }

    @Override
    public void migrate(final Connection connection) throws Exception {
    }
  }
}
