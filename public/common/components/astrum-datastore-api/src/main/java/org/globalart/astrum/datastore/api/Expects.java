
package org.globalart.astrum.datastore.api;

import java.lang.annotation.ElementType;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;

/**
 * List of {@link DataAccess} types that the current {@link DataAccess} type expects to be registered first.
 *
 * @since 3.20
 */
@Retention(RetentionPolicy.RUNTIME)
@Target(ElementType.TYPE)
public @interface Expects
{
  Class<? extends DataAccess>[] value();
}
