
package org.globalart.astrum.rest;

import java.lang.annotation.Retention;
import java.lang.annotation.Target;
import jakarta.ws.rs.NameBinding;

import static java.lang.annotation.ElementType.METHOD;
import static java.lang.annotation.ElementType.TYPE;
import static java.lang.annotation.RetentionPolicy.RUNTIME;

/**
 * Flags REST methods that should not be cached and adds the appropriate HTTP headers.
 *
 * @since 3.36
 */
@NameBinding
@Retention(RUNTIME)
@Target({TYPE, METHOD})
public @interface NotCacheable
{
}
