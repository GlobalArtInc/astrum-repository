
package org.globalart.astrum.common.stateguard;

import java.lang.annotation.ElementType;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;

import javax.annotation.Nullable;

/**
 * Marks a method as transitioning from states to state.
 *
 * Target object must implement {@link StateGuardAware}.
 *
 * @since 3.0
 */
@Retention(RetentionPolicy.RUNTIME)
@Target(ElementType.METHOD)
public @interface Transitions
{
  @Nullable
  String[] from() default {};

  String to();

  /**
   * Log interrupted transitions at debug level instead of error.
   */
  boolean silent() default false;

  /**
   * Continue to transition the state for the following exceptions.
   */
  Class<? extends Exception>[] ignore() default {};

  /**
   * Lock before transition.
   */
  boolean requiresWriteLock() default true;
}
