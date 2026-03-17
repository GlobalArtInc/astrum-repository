
package org.globalart.astrum.content.example.internal.recipe;

import java.util.Optional;

import org.globalart.astrum.content.example.ExampleContentFacet;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.matchers.token.TokenMatcher;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkState;
import static org.globalart.astrum.repository.http.HttpMethods.DELETE;
import static org.globalart.astrum.repository.http.HttpMethods.GET;
import static org.globalart.astrum.repository.http.HttpMethods.HEAD;
import static org.globalart.astrum.repository.http.HttpMethods.PUT;
import org.springframework.stereotype.Component;

/**
 * Example format content handler.
 *
 * @since 3.24
 */
@Component
public class ExampleContentHandler
    implements Handler
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  @Override
  public Response handle(final Context context) throws Exception {

    String path = contentPath(context);
    String method = context.getRequest().getAction();
    Repository repository = context.getRepository();

    log.debug("{} repository '{}' content-path: {}", method, repository.getName(), path);

    ExampleContentFacet storage = repository.facet(ExampleContentFacet.class);

    switch (method) {
      case HEAD:
      case GET: {
        Optional<Content> content = storage.get(path);
        if (content.isPresent()) {
          return HttpResponses.ok(content.get());
        }
        return HttpResponses.notFound(path);
      }

      case PUT: {
        Payload content = context.getRequest().getPayload();
        storage.put(path, content);
        return HttpResponses.created();
      }

      case DELETE: {
        boolean deleted = storage.delete(path);
        if (deleted) {
          return HttpResponses.noContent();
        }
        return HttpResponses.notFound(path);
      }

      default:
        return HttpResponses.methodNotAllowed(method, GET, HEAD, PUT, DELETE);
    }
  }

  /**
   * Pull the parsed content path out of the context.
   */
  private static String contentPath(final Context context) {
    TokenMatcher.State state = context.getAttributes().require(TokenMatcher.State.class);
    String path = state.getTokens().get("path");
    checkState(path != null, "Missing token: path");
    return path;
  }
}
