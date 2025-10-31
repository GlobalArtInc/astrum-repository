
package org.globalart.astrum.common.upgrade;

import java.lang.annotation.ElementType;
import java.lang.annotation.Inherited;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;

import jakarta.inject.Qualifier;

/**
 * Marks a class or a method as available from a specified database schema version onwards
 */
@Inherited
@Retention(RetentionPolicy.RUNTIME)
@Qualifier
@Target({ElementType.TYPE})
public @interface AvailabilityVersion
{
  String from();
}
