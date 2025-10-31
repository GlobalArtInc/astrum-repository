
package org.globalart.astrum.common.upgrade;

import java.lang.annotation.ElementType;
import java.lang.annotation.Inherited;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;

/**
 * Marks a {@link Checkpoint} that checkpoints the given model.
 * 
 * @since 3.1
 */
@Inherited
@Retention(RetentionPolicy.RUNTIME)
@Target(ElementType.TYPE)
public @interface Checkpoints
{
  String model();

  /**
   * {@code true} if the checkpoint operates on a local model, {@code false} if the model is clustered.
   */
  boolean local() default false;
}
