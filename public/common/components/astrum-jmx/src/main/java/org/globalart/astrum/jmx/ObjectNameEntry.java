
package org.globalart.astrum.jmx;

import java.lang.annotation.ElementType;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;

import javax.management.ObjectName;

import org.globalart.astrum.jmx.reflect.ManagedObject;

/**
 * Key-value entry for {@link ObjectName} customization.
 *
 * @see ManagedObject
 * @since 3.0
 */
@Retention(RetentionPolicy.RUNTIME)
@Target({ElementType.ANNOTATION_TYPE})
public @interface ObjectNameEntry
{
  /**
   * Name of object-name entry.
   */
  String name();

  /**
   * Value of object-name entry.
   */
  String value();
}
