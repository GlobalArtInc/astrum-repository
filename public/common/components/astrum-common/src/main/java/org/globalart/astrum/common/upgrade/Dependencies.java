
package org.globalart.astrum.common.upgrade;

import java.lang.annotation.ElementType;
import java.lang.annotation.Inherited;
import java.lang.annotation.Repeatable;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;

/**
 * Containing annotation that captures {@link Repeatable} {@link DependsOn} annotations.
 * 
 * @since 3.1
 */
@Inherited
@Retention(RetentionPolicy.RUNTIME)
@Target(ElementType.TYPE)
public @interface Dependencies
{
  DependsOn[] value();
}
