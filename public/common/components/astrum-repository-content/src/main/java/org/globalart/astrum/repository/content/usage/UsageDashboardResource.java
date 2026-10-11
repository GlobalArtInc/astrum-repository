package org.globalart.astrum.repository.content.usage;

import javax.annotation.Nullable;

import jakarta.ws.rs.DefaultValue;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.Produces;
import jakarta.ws.rs.QueryParam;
import jakarta.ws.rs.core.Response.Status;

import org.globalart.astrum.rest.Resource;
import org.globalart.astrum.rest.WebApplicationMessageException;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static jakarta.ws.rs.core.MediaType.APPLICATION_JSON;
import static org.apache.commons.lang3.StringUtils.isBlank;

/**
 * Serves how much of what is used in the repositories the current user may browse, for the welcome page.
 *
 * {@code GET /service/rest/internal/ui/usage-dashboard?days=30&repository=name} sums up the last {@code days} days,
 * today included, of all repositories the user may browse or of the named one; see {@link UsageDashboardService}.
 */
@Component
@Path(UsageDashboardResource.RESOURCE_PATH)
@Produces(APPLICATION_JSON)
public class UsageDashboardResource
    implements Resource
{
  static final String RESOURCE_PATH = "/internal/ui/usage-dashboard";

  private final UsageDashboardService usageDashboardService;

  @Autowired
  public UsageDashboardResource(final UsageDashboardService usageDashboardService) {
    this.usageDashboardService = checkNotNull(usageDashboardService);
  }

  @GET
  public UsageDashboardXO getUsage(
      @QueryParam("days") @DefaultValue("30") final int days,
      @QueryParam("repository") @Nullable final String repository)
  {
    return usageDashboardService.dashboard(days, isBlank(repository) ? null : repository)
        .orElseThrow(() -> new WebApplicationMessageException(Status.NOT_FOUND, "\"Repository not found\"",
            APPLICATION_JSON));
  }
}
