
package org.globalart.astrum.coreui.internal.wonderland;

import java.util.Optional;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Consumes;
import jakarta.ws.rs.POST;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.Produces;
import jakarta.ws.rs.WebApplicationException;
import jakarta.ws.rs.core.MediaType;
import jakarta.ws.rs.core.Response.Status;

import org.globalart.astrum.common.text.Strings2;
import org.globalart.astrum.common.wonderland.AuthTicketService;
import org.globalart.astrum.rest.NotCacheable;
import org.globalart.astrum.rest.Resource;

import org.apache.shiro.SecurityUtils;
import org.apache.shiro.authc.AuthenticationException;
import org.apache.shiro.authc.UsernamePasswordToken;
import org.apache.shiro.subject.Subject;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * Authenticate a user's credentials.
 *
 * @since 2.7
 */
@Component
@Path(AuthenticateResource.RESOURCE_URI)
public class AuthenticateResource
    implements Resource
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  public static final String RESOURCE_URI = "/wonderland/authenticate";

  private final AuthTicketService authTickets;

  @Autowired
  public AuthenticateResource(final AuthTicketService authTickets) {
    this.authTickets = checkNotNull(authTickets);
  }

  // FIXME: This may be missing annotation to require user or authentication annotations?

  /**
   * Authenticate a specific user and generate a one-time-use authentication token.
   *
   * @param token User authentication details.
   * @return Authentication ticket.
   */
  @POST
  @Consumes({MediaType.APPLICATION_XML, MediaType.APPLICATION_JSON})
  @Produces({MediaType.APPLICATION_XML, MediaType.APPLICATION_JSON})
  @NotCacheable
  public AuthTicketXO post(final AuthTokenXO token) {
    checkNotNull(token);

    final String username = Strings2.decodeBase64(token.getU());
    final String password = Strings2.decodeBase64(token.getP());

    // Require current user to be the requested user to authenticate
    final Subject subject = SecurityUtils.getSubject();
    final Object principal = subject.getPrincipal();
    final String principalName = principal == null ? "" : principal.toString();

    if (log.isDebugEnabled()) {
      log.debug("payload username: {}, payload password: {}, principal: {}", username, Strings2.mask(password),
          principalName);
    }

    if (!principalName.equals(username)) {
      log.warn("auth token request denied - authenticated user {} does not match payload user {}",
          principalName, username);
      throw new WebApplicationException("Username mismatch", Status.BAD_REQUEST);
    }

    // Ask the sec-manager to authenticate, this won't alter the current subject
    try {
      SecurityUtils.getSecurityManager().authenticate(new UsernamePasswordToken(username, password));
    }
    catch (AuthenticationException e) {
      log.trace("Authentication failed", e);
      throw new WebApplicationException("Authentication failed", Status.FORBIDDEN);
    }

    // At this point we should be authenticated, return a new ticket
    Optional<String> realmName = subject.getPrincipals().getRealmNames().stream().findFirst();
    return new AuthTicketXO().withT(authTickets.createTicket(username, realmName.orElse(null)));
  }
}
