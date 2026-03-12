
package org.globalart.astrum.upgrade.datastore.internal;

import java.sql.Connection;
import java.util.Arrays;
import java.util.Collection;
import java.util.stream.Collectors;

import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;
import org.globalart.astrum.upgrade.datastore.DependsOn;
import org.globalart.astrum.upgrade.datastore.RepeatableDatabaseMigrationStep;

import org.junit.Before;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.junit.Assert.assertThrows;
import static org.junit.Assert.assertTrue;

public class SimpleDependencyResolverTest
{
  private final One one = new One();

  private final Two two = new Two();

  private final Three three = new Three();

  private final TestMigrationStep versioned = new TestMigrationStep();

  @Before
  public void setup() {
    assertTrue("Sanity check we expect a versioned step", versioned.version().isPresent());
  }

  @Test
  public void testOrderUpgrades() {
    // we intentionally request ordering with the steps in the wrong order
    assertThat(orderUpgrades(three, one, two, versioned),
        contains("TestMigrationStep", "One", "Z_001_Two", "Z_002_Three"));

    // Request an ordering where a dependency (one) is missing
    assertThrows(IllegalStateException.class, () -> orderUpgrades(three, two, versioned));
  }

  private Collection<String> orderUpgrades(final DatabaseMigrationStep... steps) {
    return new SimpleDependencyResolver(Arrays.asList(steps)).resolve()
        .stream()
        .map(AstrumJavaMigration::getDescription)
        .collect(Collectors.toList());
  }

  private static class One
      extends TestRepeatableDatabaseMigrationStep
  {
  }

  @DependsOn(One.class)
  private static class Two
      extends TestRepeatableDatabaseMigrationStep
  {
  }

  @DependsOn(One.class)
  @DependsOn(Two.class)
  private static class Three
      extends TestRepeatableDatabaseMigrationStep
  {
    // A step with multiple dependencies
  }

  private abstract static class TestRepeatableDatabaseMigrationStep
      implements RepeatableDatabaseMigrationStep
  {
    @Override
    public void migrate(final Connection connection) throws Exception {
      // not applicable
    }

    @Override
    public Integer getChecksum() {
      return 1;
    }
  }
}
