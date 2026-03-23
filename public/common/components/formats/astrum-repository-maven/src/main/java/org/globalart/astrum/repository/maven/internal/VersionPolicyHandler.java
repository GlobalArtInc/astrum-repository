
package org.globalart.astrum.repository.maven.internal;

import javax.annotation.Nonnull;
import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.maven.MavenFacet;
import org.globalart.astrum.repository.maven.MavenPath;
import org.globalart.astrum.repository.maven.MavenPath.Coordinates;
import org.globalart.astrum.repository.maven.VersionPolicy;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Response;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.http.HttpMethods.GET;
import static org.globalart.astrum.repository.http.HttpMethods.HEAD;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

/**
 * Enforces the repository's version policy on all requests.
 *
 * @since 3.25
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class VersionPolicyHandler
    implements Handler
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final VersionPolicyValidator versionPolicyValidator;

  @Autowired
  public VersionPolicyHandler(final VersionPolicyValidator versionPolicyValidator) {
    this.versionPolicyValidator = checkNotNull(versionPolicyValidator);
  }

  @Nonnull
  @Override
  public Response handle(@Nonnull final Context context) throws Exception {
    final MavenPath path = context.getAttributes().require(MavenPath.class);
    final MavenFacet mavenFacet = context.getRepository().facet(MavenFacet.class);
    final VersionPolicy versionPolicy = mavenFacet.getVersionPolicy();
    final Coordinates coordinates = path.getCoordinates();
    if (coordinates != null && !versionPolicyValidator.validArtifactPath(versionPolicy, coordinates)) {
      return createResponse(context,
          "Repository version policy: " + versionPolicy + " does not allow version: " + coordinates.getVersion());
    }
    if (!versionPolicyValidator.validMetadataPath(versionPolicy, path.main().getPath())) {
      return createResponse(context,
          "Repository version policy: " + versionPolicy + " does not allow metadata in path: " + path.getPath());
    }
    return context.proceed();
  }

  private static Response createResponse(final Context context, final String message) {
    switch (context.getRequest().getAction()) {
      case GET:
      case HEAD:
        return HttpResponses.notFound(message);
      default:
        return HttpResponses.badRequest(message);
    }
  }
}
