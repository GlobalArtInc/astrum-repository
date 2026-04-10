
package org.globalart.astrum.repository.apt.datastore.internal.proxy;

import java.util.regex.Matcher;
import java.util.regex.Pattern;

import org.globalart.astrum.repository.apt.datastore.AptContentFacet;
import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Response;

import org.apache.commons.lang3.StringUtils;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Handler that validates requests against the configured distribution when requireDistribution is enabled.
 * When enabled, requests for distributions other than the configured one will return 404.
 */
@Component
public class AptDistributionValidationHandler
    implements Handler
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private static final Pattern DISTS_PATH_PATTERN = Pattern.compile("^/?dists/([^/]+)/");

  @Override
  public Response handle(final Context context) throws Exception {
    AptContentFacet contentFacet = context.getRepository().facet(AptContentFacet.class);

    if (!contentFacet.isEnforceDistribution()) {
      return context.proceed();
    }

    String configuredDistribution = contentFacet.getDistribution();
    if (StringUtils.isBlank(configuredDistribution)) {
      log.error("Enforce Distribution is enabled but no distribution is configured for repository '{}'. " +
          "All requests will be allowed.", context.getRepository().getName());
      return context.proceed();
    }

    String path = context.getRequest().getPath();
    Matcher matcher = DISTS_PATH_PATTERN.matcher(path);

    if (matcher.find()) {
      String requestedDistribution = matcher.group(1);
      if (!configuredDistribution.equals(requestedDistribution)) {
        log.debug("Rejecting request for distribution '{}', configured distribution is '{}'",
            requestedDistribution, configuredDistribution);
        return HttpResponses.notFound("Distribution not available");
      }
    }

    return context.proceed();
  }
}
