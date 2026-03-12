
package org.globalart.astrum.upgrade.datastore.internal;

import java.util.Collection;
import java.util.HashMap;
import java.util.Iterator;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Map.Entry;
import java.util.function.Function;
import java.util.stream.Collectors;
import java.util.stream.Stream;

import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;
import org.globalart.astrum.upgrade.datastore.DependsOn;
import org.globalart.astrum.upgrade.datastore.RepeatableDatabaseMigrationStep;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * This resolver will return a new collection which contains the steps in order of {@link DependsOn}.
 *
 * 1. Steps which provide a version
 * 2. RepeatableSteps which do not define a {@link @DependsOn}
 * 3. RepeatableSteps based on their depends on ordering
 */
class SimpleDependencyResolver
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final Collection<DatabaseMigrationStep> steps;

  SimpleDependencyResolver(final Collection<DatabaseMigrationStep> steps) {
    this.steps = checkNotNull(steps);
  }

  /**
   * @throws IllegalStateException if there is no solution to the dependency resolution
   */
  Collection<AstrumJavaMigration> resolve() {
    // LinkedHashMap to provide ordering
    Map<Class<? extends DatabaseMigrationStep>, AstrumJavaMigration> orderedMap = new LinkedHashMap<>();

    // Add versioned steps, the dependencies are managed by Flyway
    steps.stream()
        .filter(step -> step.version().isPresent())
        .forEach(step -> orderedMap.put(step.getClass(), new AstrumJavaMigration(step)));

    // Add steps without dependencies, we don't specify a 'round' for these
    steps.stream()
        .filter(step -> !step.version().isPresent())
        .filter(step -> dependencies(step).isEmpty())
        .forEach(step -> orderedMap.put(step.getClass(), new AstrumJavaMigration(step)));

    // Create a map containing Step -> Dependencies
    Map<DatabaseMigrationStep, List<Class<? extends RepeatableDatabaseMigrationStep>>> stepToDependency = steps.stream()
        .filter(step -> !orderedMap.containsKey(step.getClass()))
        .collect(Collectors.toMap(Function.identity(), SimpleDependencyResolver::dependencies));

    // We process the rest in 'rounds' as Flyway uses the description to order versionless migrations.
    // Steps within a round cannot depend on each other.
    int round = 1;
    boolean found;
    do {
      found = false;

      Iterator<Entry<DatabaseMigrationStep, List<Class<? extends RepeatableDatabaseMigrationStep>>>> iter =
          stepToDependency
              .entrySet()
              .iterator();

      Map<Class<? extends DatabaseMigrationStep>, AstrumJavaMigration> roundMap = new HashMap<>();

      while (iter.hasNext()) {
        Entry<DatabaseMigrationStep, List<Class<? extends RepeatableDatabaseMigrationStep>>> entry = iter.next();

        if (orderedMap.keySet().containsAll(entry.getValue())) {
          // If all dependencies are met, add the step to the orderedMap and remove from future evaluations
          found = true;
          iter.remove();
          roundMap.put(entry.getKey().getClass(), new AstrumJavaMigration(entry.getKey(), round));
        }
      }
      // Finally add the steps to a round
      orderedMap.putAll(roundMap);
      round++;
    }
    while (found);

    if (!stepToDependency.isEmpty()) {
      log.error("Unable to compute dependencies for: {}", stepToDependency.keySet());
      throw new IllegalStateException("Unable to compute dependencies between upgrades");
    }

    return orderedMap.values();
  }

  /*
   * Retrieve the classes specified via @DependsOn annotations
   */
  private static List<Class<? extends RepeatableDatabaseMigrationStep>> dependencies(final DatabaseMigrationStep step) {
    return Stream.of(step.getClass().getAnnotationsByType(DependsOn.class))
        .map(DependsOn::value)
        .collect(Collectors.toList());
  }

}
