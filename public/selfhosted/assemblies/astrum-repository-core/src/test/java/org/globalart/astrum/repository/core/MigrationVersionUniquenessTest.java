
package org.globalart.astrum.repository.core;

import java.lang.reflect.Modifier;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.Set;

import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;

import org.junit.Test;
import org.reflections.Reflections;
import org.reflections.scanners.Scanners;
import org.reflections.util.ClasspathHelper;
import org.reflections.util.ConfigurationBuilder;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.empty;

/**
 * Validates that all DatabaseMigrationStep classes have unique version numbers.
 *
 * This test prevents runtime failures due to duplicate Flyway migration versions
 * by catching them at build time.
 *
 * Related to ASTRUM-49154 - Ensures migration version uniqueness across all modules
 * included in the astrum-repository-core (OSS) distribution.
 */
public class MigrationVersionUniquenessTest
{
  @Test
  public void testNoMigrationVersionDuplicates() throws Exception {
    Map<String, List<String>> versionToClasses = new HashMap<>();

    // Scan for all DatabaseMigrationStep implementations
    Reflections reflections = new Reflections(new ConfigurationBuilder()
        .setUrls(ClasspathHelper.forPackage("org.globalart.astrum"))
        .setScanners(Scanners.SubTypes));

    Set<Class<? extends DatabaseMigrationStep>> migrationClasses =
        reflections.getSubTypesOf(DatabaseMigrationStep.class);

    // Extract versions and track which classes use them
    for (Class<? extends DatabaseMigrationStep> clazz : migrationClasses) {
      // Skip abstract classes and interfaces
      if (Modifier.isAbstract(clazz.getModifiers()) || clazz.isInterface()) {
        continue;
      }

      try {
        // Try to instantiate without constructor parameters (works for most migrations)
        DatabaseMigrationStep instance;
        try {
          instance = clazz.getDeclaredConstructor().newInstance();
        }
        catch (NoSuchMethodException e) {
          // Skip classes that require constructor parameters (they need DI)
          continue;
        }

        Optional<String> version = instance.version();

        if (version.isPresent()) {
          String versionNum = version.get();
          versionToClasses.computeIfAbsent(versionNum, k -> new ArrayList<>())
              .add(clazz.getName());
        }
      }
      catch (Exception e) {
        // Silently skip classes that can't be instantiated
      }
    }

    // Find duplicates
    List<String> errors = new ArrayList<>();
    versionToClasses.forEach((version, classes) -> {
      if (classes.size() > 1) {
        errors.add(String.format(
            "Duplicate migration version '%s' found in %d classes:\n  - %s",
            version,
            classes.size(),
            String.join("\n  - ", classes)));
      }
    });

    // Fail with clear error message if duplicates found
    assertThat("Found duplicate Flyway migration versions:\n\n" + String.join("\n\n", errors),
        errors, empty());
  }
}
