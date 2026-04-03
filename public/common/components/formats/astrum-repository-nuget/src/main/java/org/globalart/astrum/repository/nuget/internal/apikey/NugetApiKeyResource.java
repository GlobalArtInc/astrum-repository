package org.globalart.astrum.repository.nuget.internal.apikey;

import java.nio.charset.StandardCharsets;
import java.util.Base64;
import java.util.Map;

import jakarta.ws.rs.DELETE;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.Produces;
import jakarta.ws.rs.QueryParam;
import jakarta.ws.rs.core.Response.Status;

import org.globalart.astrum.common.wonderland.AuthTicketService;
import org.globalart.astrum.rest.Resource;
import org.globalart.astrum.rest.WebApplicationMessageException;
import org.globalart.astrum.security.SecurityHelper;
import org.globalart.astrum.security.authc.apikey.ApiKey;
import org.globalart.astrum.security.authc.apikey.ApiKeyService;

import org.apache.shiro.authz.annotation.RequiresAuthentication;
import org.apache.shiro.authz.annotation.RequiresPermissions;
import org.apache.shiro.authz.annotation.RequiresUser;
import org.apache.shiro.subject.PrincipalCollection;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static jakarta.ws.rs.core.MediaType.APPLICATION_JSON;

@Component
@Path(NugetApiKeyResource.RESOURCE_PATH)
@Produces(APPLICATION_JSON)
public class NugetApiKeyResource
    implements Resource
{
  public static final String RESOURCE_PATH = "internal/nuget-api-key";

  private final ApiKeyService apiKeyService;

  private final SecurityHelper securityHelper;

  private final AuthTicketService authTickets;

  @Autowired
  public NugetApiKeyResource(
      final ApiKeyService apiKeyService,
      final SecurityHelper securityHelper,
      final AuthTicketService authTickets)
  {
    this.apiKeyService = checkNotNull(apiKeyService);
    this.securityHelper = checkNotNull(securityHelper);
    this.authTickets = checkNotNull(authTickets);
  }

  @GET
  @RequiresUser
  @RequiresAuthentication
  @RequiresPermissions("astrum:apikey:read")
  public Map<String, String> readKey(@QueryParam("authToken") final String authToken) {
    redeem(authToken);
    PrincipalCollection principals = securityHelper.subject().getPrincipals();
    char[] apiKey = apiKeyService.getApiKey(NugetApiKeyRealm.NAME, principals)
        .map(ApiKey::getApiKey)
        .orElseGet(() -> apiKeyService.createApiKey(NugetApiKeyRealm.NAME, principals));
    return Map.of("apiKey", new String(apiKey));
  }

  @DELETE
  @RequiresUser
  @RequiresAuthentication
  @RequiresPermissions("astrum:apikey:delete")
  public void resetKey(@QueryParam("authToken") final String authToken) {
    redeem(authToken);
    apiKeyService.deleteApiKey(NugetApiKeyRealm.NAME, securityHelper.subject().getPrincipals());
  }

  private void redeem(final String authToken) {
    if (authToken == null || authToken.isBlank()) {
      throw new WebApplicationMessageException(Status.FORBIDDEN, "Missing authentication ticket");
    }
    String ticket;
    try {
      ticket = new String(Base64.getDecoder().decode(authToken.trim()), StandardCharsets.UTF_8);
    }
    catch (IllegalArgumentException e) {
      ticket = authToken.trim();
    }
    if (!authTickets.redeemTicket(ticket)) {
      throw new WebApplicationMessageException(Status.FORBIDDEN, "Invalid authentication ticket");
    }
  }
}
