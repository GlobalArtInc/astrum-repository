
package org.globalart.astrum.repository.view.handlers;

import java.util.Optional;

import javax.annotation.Nonnull;

import org.globalart.astrum.repository.http.HttpConditions;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.ViewFacet;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Primary;
import org.springframework.context.annotation.Scope;

import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static org.globalart.astrum.repository.http.HttpConditions.makeConditional;
import static org.globalart.astrum.repository.http.HttpConditions.makeUnconditional;
import static org.globalart.astrum.repository.http.HttpMethods.DELETE;
import static org.globalart.astrum.repository.http.HttpMethods.GET;
import static org.globalart.astrum.repository.http.HttpMethods.HEAD;
import static org.globalart.astrum.repository.http.HttpMethods.POST;
import static org.globalart.astrum.repository.http.HttpMethods.PUT;

/**
 * A format-neutral handler for conditional requests. It relies on existence of following HTTP entity headers:
 * <ul>
 * <li>Last-Modified</li>
 * <li>ETag</li>
 * </ul>
 * There is a handy {@link ContentHeadersHandler} handler that adds these for you, if you are serving up {@link
 * Content} payloads properly decorated. If not, you should have them somehow (format specific way) set on {@link
 * Response} instances coming from your {@link Handler}s.
 *
 * @see ContentHeadersHandler
 * @see Content
 * @since 3.0
 */
@Primary
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class ConditionalRequestHandler
    implements Handler
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  @Nonnull
  @Override
  public Response handle(@Nonnull final Context context) throws Exception {
    if (HttpConditions.isConditional(context.getRequest())) {
      makeUnconditional(context.getRequest());
      try {
        return handleConditional(context);
      }
      finally {
        makeConditional(context.getRequest());
      }
    }

    return context.proceed();
  }

  @Nonnull
  private Response handleConditional(@Nonnull final Context context) throws Exception {
    final String action = context.getRequest().getAction();
    log.debug("Conditional request: {} {}",
        action,
        context.getRequest().getPath());
    switch (action) {
      case GET:
      case HEAD: {
        final Response response = context.proceed();
        if (response.getStatus().isSuccessful()) {
          // copy only ETag header, leave out all other entity headers
          Optional<Response> conditionalResponse = HttpConditions.maybeCreateConditionalResponse(context, response);
          return conditionalResponse.orElse(response);
        }
        else {
          return response;
        }
      }

      case POST:
      case PUT:
      case DELETE: {
        final Request getRequest = new Request.Builder().copy(context.getRequest()).action(GET).build();
        final Response response = context.getRepository().facet(ViewFacet.class).dispatch(getRequest);
        if (response.getStatus().isSuccessful()) {
          Optional<Response> conditionalResponse = HttpConditions.maybeCreateConditionalResponse(context, response);
          if (conditionalResponse.isPresent()) {
            return conditionalResponse.get();
          }
        }
        return context.proceed();
      }
      default:
        return context.proceed();
    }
  }
}
