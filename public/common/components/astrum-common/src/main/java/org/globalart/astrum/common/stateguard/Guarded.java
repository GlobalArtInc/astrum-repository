
package org.globalart.astrum.common.stateguard;

import java.lang.annotation.ElementType;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;

/**
 * Marks a method as guarded by one or more states.
 *
 * Target object must implement {@link StateGuardAware}.
 *
 * @since 3.0
 */
@Retention(RetentionPolicy.RUNTIME)
@Target(ElementType.METHOD)
public @interface Guarded
{
  String[] by();
}
