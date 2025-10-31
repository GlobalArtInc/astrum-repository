
package org.globalart.astrum.common.upgrade;

import java.lang.annotation.ElementType;
import java.lang.annotation.Inherited;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;

/**
 * Marks an {@link Upgrade} that upgrades the given model from version "from" to version "to".
 * 
 * @since 3.1
 */
@Inherited
@Retention(RetentionPolicy.RUNTIME)
@Target(ElementType.TYPE)
public @interface Upgrades
{
  String model();

  String from();

  String to();
}
