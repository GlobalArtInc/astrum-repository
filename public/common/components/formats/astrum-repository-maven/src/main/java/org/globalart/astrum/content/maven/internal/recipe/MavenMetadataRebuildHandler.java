
package org.globalart.astrum.content.maven.internal.recipe;

import javax.annotation.Nonnull;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.maven.MavenMetadataRebuildFacet;
import org.globalart.astrum.repository.types.ProxyType;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Response;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static org.apache.commons.lang3.StringUtils.prependIfMissing;
import static org.globalart.astrum.repository.http.HttpMethods.GET;
import static org.globalart.astrum.repository.http.HttpMethods.HEAD;
import org.springframework.stereotype.Component;

/**
 * @since 3.26
 */
@Component
public class MavenMetadataRebuildHandler
    implements Handler
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private static final String PATH_PREFIX = "/";

  @Nonnull
  @Override
  public Response handle(@Nonnull final Context context) throws Exception {
    String method = context.getRequest().getAction();
    Repository repository = context.getRepository();
    if ((GET.equals(method) || HEAD.equals(method)) && isNotProxy(repository)) {
      repository.facet(MavenMetadataRebuildFacet.class)
          .maybeRebuildMavenMetadata(prependIfMissing(context.getRequest().getPath(), PATH_PREFIX), false, true);
    }
    return context.proceed();
  }

  protected boolean isNotProxy(@Nonnull final Repository repository) {
    return !ProxyType.NAME.equals(repository.getType().getValue());
  }
}
