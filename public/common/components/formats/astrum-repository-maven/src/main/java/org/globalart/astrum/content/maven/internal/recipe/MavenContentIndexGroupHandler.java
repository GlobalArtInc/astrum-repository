
package org.globalart.astrum.content.maven.internal.recipe;

import javax.annotation.Nonnull;

import org.globalart.astrum.content.maven.MavenContentFacet;
import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.maven.MavenPath;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Response;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;

import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Group specific handler of Maven indexes: it serves up merged group index, if present.
 *
 * @since 3.26
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
class MavenContentIndexGroupHandler
    implements Handler
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  @Nonnull
  @Override
  public Response handle(@Nonnull final Context context) throws Exception {
    MavenPath mavenPath = context.getAttributes().require(MavenPath.class);
    MavenContentFacet mavenContentFacet = context.getRepository().facet(MavenContentFacet.class);
    return mavenContentFacet.get(mavenPath)
        .map(HttpResponses::ok)
        .orElseGet(() -> HttpResponses.notFound(mavenPath.getPath()));
  }
}
