
package org.globalart.astrum.upgrade.datastore;

import java.lang.annotation.ElementType;
import java.lang.annotation.Inherited;
import java.lang.annotation.Repeatable;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;

/**
 * Marks a dependency from an {@link RepeatableDatabaseMigrationStep} to another repeatable step.
 *
 * Each {@link RepeatableDatabaseMigrationStep} can have zero to many explicit {@link DependsOn}.
 */
@Inherited
@Repeatable(Dependencies.class)
@Retention(RetentionPolicy.RUNTIME)
@Target(ElementType.TYPE)
public @interface DependsOn
{
  Class<? extends RepeatableDatabaseMigrationStep> value();
}
