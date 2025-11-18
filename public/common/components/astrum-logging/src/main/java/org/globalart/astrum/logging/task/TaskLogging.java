
package org.globalart.astrum.logging.task;

import java.lang.annotation.ElementType;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;

import static org.globalart.astrum.logging.task.TaskLogType.BOTH;

/**
 * Annotation to be applied to a TaskSupport class to indicate what type of task logging should happen.
 *
 * @since 3.5
 */
@Retention(RetentionPolicy.RUNTIME)
@Target(ElementType.TYPE)
public @interface TaskLogging
{
  TaskLogType value() default BOTH;
}
