
package org.globalart.astrum.content.maven.internal.recipe;

import java.io.IOException;

import javax.annotation.Nonnull;
import org.globalart.astrum.content.maven.MavenContentFacet;
import org.globalart.astrum.repository.IllegalOperationException;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.maven.LayoutPolicy;
import org.globalart.astrum.repository.maven.MavenPath;
import org.globalart.astrum.repository.maven.MavenPath.Coordinates;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Response;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static org.globalart.astrum.repository.http.HttpMethods.DELETE;
import static org.globalart.astrum.repository.http.HttpMethods.GET;
import static org.globalart.astrum.repository.http.HttpMethods.HEAD;
import static org.globalart.astrum.repository.http.HttpMethods.PUT;
import org.springframework.stereotype.Component;

/**
 * @since 3.25
 */
@Component
public class MavenContentHandler
    implements Handler
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  @Override
  public Response handle(@Nonnull final Context context) throws Exception {
    MavenPath mavenPath = contentPath(context);
    String path = mavenPath.getPath();
    String method = context.getRequest().getAction();
    Repository repository = context.getRepository();

    log.debug("{} repository '{}' content-path: {}", method, repository.getName(), path);

    MavenContentFacet storage = repository.facet(MavenContentFacet.class);

    switch (method) {
      case HEAD:
      case GET:
        return doGet(mavenPath, storage);

      case PUT:
        doPut(context, mavenPath, storage);
        return HttpResponses.created();

      case DELETE:
        return doDelete(mavenPath, storage);

      default:
        return HttpResponses.methodNotAllowed(method, GET, HEAD, PUT, DELETE);
    }
  }

  private MavenPath contentPath(@Nonnull final Context context) {
    return context.getAttributes().require(MavenPath.class);
  }

  private Response doGet(final MavenPath mavenPath, final MavenContentFacet storage) throws IOException {
    return storage
        .get(mavenPath)
        .map(HttpResponses::ok)
        .orElseGet(() -> HttpResponses.notFound(mavenPath.getPath()));
  }

  private void doPut(
      @Nonnull final Context context,
      final MavenPath mavenPath,
      final MavenContentFacet storage) throws IOException
  {
    validatePathForStrictLayoutPolicy(mavenPath, storage);
    storage.put(mavenPath, context.getRequest().getPayload());
  }

  private void validatePathForStrictLayoutPolicy(final MavenPath mavenPath, final MavenContentFacet storage) {
    if (storage.layoutPolicy() == LayoutPolicy.STRICT
        && isValidSnapshot(mavenPath.getCoordinates())
        && !storage.getMavenPathParser().isRepositoryMetadata(mavenPath)) {
      throw new IllegalOperationException(" Invalid mavenPath for a Maven 2 repository");
    }
  }

  private Response doDelete(final MavenPath mavenPath, final MavenContentFacet storage) throws IOException {
    boolean deleted = storage.delete(mavenPath);
    if (deleted) {
      return HttpResponses.noContent();
    }
    return HttpResponses.notFound(mavenPath.getPath());
  }

  private boolean isValidSnapshot(Coordinates coordinates) {
    return coordinates == null || (coordinates.isSnapshot() &&
        !coordinates.getVersion().equals(coordinates.getBaseVersion()) &&
        (coordinates.getTimestamp() == null || coordinates.getBuildNumber() == null));
  }
}
