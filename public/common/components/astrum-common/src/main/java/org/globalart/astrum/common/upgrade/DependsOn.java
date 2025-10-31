
package org.globalart.astrum.common.upgrade;

import java.lang.annotation.ElementType;
import java.lang.annotation.Inherited;
import java.lang.annotation.Repeatable;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;

/**
 * Marks a dependency from an {@link Upgrade} to another model/version.
 * 
 * Each {@link Upgrade} can have zero to many explicit {@link DependsOn}.
 * It also has an implicit dependency to the model/version it upgrades.
 * 
 * @since 3.1
 */
@Inherited
@Repeatable(Dependencies.class)
@Retention(RetentionPolicy.RUNTIME)
@Target(ElementType.TYPE)
public @interface DependsOn
{
  String model();

  String version();

  /**
   * Set to true if you also change content owned by this dependency during the upgrade.
   *
   * @since 3.13
   */
  boolean checkpoint() default false;
}
