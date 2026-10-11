package org.globalart.astrum.accesstoken.internal;

import java.time.OffsetDateTime;
import java.time.ZoneOffset;
import java.util.Collection;
import java.util.List;

import jakarta.ws.rs.Consumes;
import jakarta.ws.rs.DELETE;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.POST;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.PathParam;
import jakarta.ws.rs.Produces;
import jakarta.ws.rs.QueryParam;
import jakarta.ws.rs.core.Response.Status;

import org.globalart.astrum.rest.Resource;
import org.globalart.astrum.rest.WebApplicationMessageException;
import org.globalart.astrum.security.accesstoken.AccessTokenPrincipal;
import org.globalart.astrum.security.accesstoken.AccessTokenScope;
import org.globalart.astrum.security.anonymous.AnonymousHelper;

import org.apache.shiro.SecurityUtils;
import org.apache.shiro.authz.annotation.RequiresAuthentication;
import org.apache.shiro.authz.annotation.RequiresUser;
import org.apache.shiro.subject.Subject;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static jakarta.ws.rs.core.MediaType.APPLICATION_JSON;

/**
 * Lets users manage their own access tokens, and administrators see and revoke everyone's.
 *
 * Tokens cannot be managed with an access token, only from a signed-in session.
 */
@Component
@Consumes(APPLICATION_JSON)
@Produces(APPLICATION_JSON)
@Path(AccessTokenResource.RESOURCE_PATH)
public class AccessTokenResource
    implements Resource
{
  static final String RESOURCE_PATH = "internal/ui/access-tokens";

  static final String READ_ALL_PERMISSION = "astrum:accesstokens:read";

  static final String DELETE_ANY_PERMISSION = "astrum:accesstokens:delete";

  private final AccessTokenService tokens;

  @Autowired
  public AccessTokenResource(final AccessTokenService tokens) {
    this.tokens = checkNotNull(tokens);
  }

  @GET
  @RequiresUser
  @RequiresAuthentication
  public List<AccessTokenXO> list(@QueryParam("all") final boolean all) {
    Subject subject = signedInSubject();
    Collection<AccessTokenData> found;
    if (all) {
      if (!subject.isPermitted(READ_ALL_PERMISSION)) {
        throw error(Status.FORBIDDEN, "You may not see other users' access tokens");
      }
      found = tokens.listAll();
    }
    else {
      found = tokens.list(username(subject));
    }
    OffsetDateTime now = now();
    return found.stream().map(token -> AccessTokenXO.from(token, now)).toList();
  }

  @POST
  @RequiresUser
  @RequiresAuthentication
  public CreatedAccessTokenXO create(final CreateAccessTokenXO request) {
    Subject subject = signedInSubject();
    if (request == null) {
      throw error(Status.BAD_REQUEST, "Token name is required");
    }
    AccessTokenScope scope;
    try {
      scope = AccessTokenScope.fromId(request.scope() == null ? AccessTokenScope.READ.id() : request.scope());
    }
    catch (IllegalArgumentException e) {
      throw error(Status.BAD_REQUEST, e.getMessage());
    }
    try {
      AccessTokenService.CreatedAccessToken created =
          tokens.create(subject.getPrincipals(), request.name(), scope, request.expiresInDays());
      return new CreatedAccessTokenXO(AccessTokenXO.from(created.token(), now()), created.secret());
    }
    catch (AccessTokenException e) {
      throw error(Status.BAD_REQUEST, e.getMessage());
    }
  }

  @DELETE
  @Path("{id}")
  @RequiresUser
  @RequiresAuthentication
  public void revoke(@PathParam("id") final String id) {
    Subject subject = signedInSubject();
    AccessTokenData token = tokens.read(id).orElseThrow(() -> error(Status.NOT_FOUND, "Access token not found"));
    boolean own = token.getUsername().equals(username(subject));
    if (!own && !subject.isPermitted(DELETE_ANY_PERMISSION)) {
      throw error(Status.NOT_FOUND, "Access token not found");
    }
    tokens.revoke(token);
  }

  private static Subject signedInSubject() {
    Subject subject = SecurityUtils.getSubject();
    if (subject == null || subject.getPrincipal() == null || AnonymousHelper.isAnonymous(subject)) {
      throw error(Status.FORBIDDEN, "Sign in to manage access tokens");
    }
    if (!subject.getPrincipals().byType(AccessTokenPrincipal.class).isEmpty()) {
      throw error(Status.FORBIDDEN, "Access tokens cannot be managed with an access token");
    }
    return subject;
  }

  private static String username(final Subject subject) {
    return subject.getPrincipal().toString();
  }

  private static OffsetDateTime now() {
    return OffsetDateTime.now(ZoneOffset.UTC);
  }

  private static WebApplicationMessageException error(final Status status, final String message) {
    return new WebApplicationMessageException(status, message, APPLICATION_JSON);
  }
}
