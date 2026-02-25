
package org.globalart.astrum.repository.security;

import javax.annotation.Nonnull;
import javax.annotation.Nullable;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Response;

import com.google.common.annotations.VisibleForTesting;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Security handler that ensures the requesting user has permission to access repository content.
 * <p>
 * Creates and manages a request-scoped cache for selector evaluations to improve performance
 * for Docker digest pulls and other scenarios that cause repeated selector evaluations (ASTRUM-50181).
 * </p>
 *
 * @since 3.0
 */
@Component
public class SecurityHandler
    implements org.globalart.astrum.repository.view.handlers.SecurityHandler
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  @VisibleForTesting
  static final String AUTHORIZED_KEY = "security.authorized";

  /**
   * Key for storing the request-scoped selector evaluation cache.
   */
  @VisibleForTesting
  static final String SELECTOR_CACHE_KEY = "security.selectorCache";

  private final Handler loginsCounterHandler;

  @Autowired
  public SecurityHandler(
      @Qualifier("astrum.analytics.loginsCounterHandler") @Nullable final Handler loginsCounterHandler)
  {
    this.loginsCounterHandler = loginsCounterHandler;
  }

  @Nonnull
  @Override
  public Response handle(@Nonnull final Context context) throws Exception {
    SecurityFacet securityFacet = context.getRepository().facet(SecurityFacet.class);

    // Create request-scoped cache for selector evaluations
    SelectorEvaluationCache selectorCache = new SelectorEvaluationCache();
    context.getAttributes().set(SELECTOR_CACHE_KEY, selectorCache);

    try {
      // we employ the model that one security check per request is all that is necessary, if this handler is in a
      // nested repository (because this is a group repository), there is no need to check authz again
      if (context.getAttributes().get(AUTHORIZED_KEY) == null) {
        securityFacet.ensurePermitted(context.getRequest(), selectorCache);
        context.getAttributes().set(AUTHORIZED_KEY, true);
        if (loginsCounterHandler != null) {
          context.insertHandler(loginsCounterHandler);
        }
      }

      return context.proceed();
    }
    finally {
      // Clear request-scoped selector evaluation cache to prevent memory leaks
      selectorCache.clear();
      context.getAttributes().remove(SELECTOR_CACHE_KEY);
    }
  }
}
