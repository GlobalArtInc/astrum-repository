
package org.globalart.astrum.siesta.internal.resteasy;

import java.io.IOException;

import jakarta.ws.rs.container.ContainerRequestContext;
import jakarta.ws.rs.container.ContainerResponseContext;
import jakarta.ws.rs.container.ContainerResponseFilter;
import jakarta.ws.rs.core.CacheControl;

import org.globalart.astrum.rest.NotCacheable;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static jakarta.ws.rs.core.HttpHeaders.CACHE_CONTROL;
import static jakarta.ws.rs.core.HttpHeaders.EXPIRES;

/**
 * @since 3.36
 */
@NotCacheable
public class NotCacheableResponseFilter
    implements ContainerResponseFilter
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private static final String PRAGMA = "Pragma";

  private static final String NO_CACHE = "no-cache";

  @Override
  public void filter(
      final ContainerRequestContext requestContext,
      final ContainerResponseContext responseContext) throws IOException
  {
    CacheControl cacheControl = new CacheControl();
    cacheControl.setNoStore(true);
    cacheControl.setNoCache(true);
    cacheControl.setMustRevalidate(true);
    cacheControl.setProxyRevalidate(true);
    cacheControl.setMaxAge(0);
    cacheControl.setSMaxAge(0);
    responseContext.getHeaders().add(CACHE_CONTROL, cacheControl.toString());
    responseContext.getHeaders().add(EXPIRES, 0);
    responseContext.getHeaders().add(PRAGMA, NO_CACHE);
  }
}
