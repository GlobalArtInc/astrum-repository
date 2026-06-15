
package org.globalart.astrum.coreui;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.validation.Valid;
import jakarta.validation.constraints.NotNull;
import jakarta.ws.rs.Consumes;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.PUT;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.PathParam;
import jakarta.ws.rs.Produces;
import jakarta.ws.rs.core.Response.Status;

import org.globalart.astrum.common.wonderland.AuthTicketService;
import org.globalart.astrum.rest.Resource;
import org.globalart.astrum.rest.WebApplicationMessageException;
import org.globalart.astrum.security.SecuritySystem;
import org.globalart.astrum.security.anonymous.AnonymousConfiguration;
import org.globalart.astrum.security.anonymous.AnonymousManager;
import org.globalart.astrum.security.user.NoSuchUserManagerException;
import org.globalart.astrum.security.user.User;
import org.globalart.astrum.security.user.UserNotFoundException;
import org.globalart.astrum.validation.Validate;

import org.apache.shiro.authz.annotation.RequiresAuthentication;
import org.apache.shiro.authz.annotation.RequiresPermissions;
import org.apache.shiro.authz.annotation.RequiresUser;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static jakarta.ws.rs.core.MediaType.APPLICATION_JSON;
import static org.globalart.astrum.security.user.UserManager.DEFAULT_SOURCE;
import org.springframework.stereotype.Component;

/**
 * User resource.
 *
 * @since 3.24
 */
@Component
@Consumes(APPLICATION_JSON)
@Produces(APPLICATION_JSON)
@Path(UserResource.RESOURCE_PATH)
public class UserResource
    implements Resource
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  static final String RESOURCE_PATH = "internal/ui/user";

  private final SecuritySystem securitySystem;

  private final AuthTicketService authTickets;

  private final AnonymousManager anonymousManager;

  @Autowired
  public UserResource(
      final SecuritySystem securitySystem,
      final AuthTicketService authTickets,
      final AnonymousManager anonymousManager)
  {
    this.securitySystem = checkNotNull(securitySystem);
    this.authTickets = checkNotNull(authTickets);
    this.anonymousManager = checkNotNull(anonymousManager);
  }

  /**
   * Retrieves user account (logged in user info).
   *
   * @return current logged in user account.
   */
  @GET
  @RequiresUser
  public UserAccountXO readAccount() {
    return convert(getCurrentUser());
  }

  @PUT
  @RequiresUser
  @RequiresAuthentication
  @Validate
  public void updateAccount(
      @NotNull @Valid final UserAccountXO xo) throws UserNotFoundException, NoSuchUserManagerException
  {
    User user = getCurrentUser();
    if (!user.getUserId().equals(xo.getUserId())) {
      throw new WebApplicationMessageException(Status.BAD_REQUEST,
          "Mismatch between authenticated user and user to update.");
    }
    user.setFirstName(xo.getFirstName());
    user.setLastName(xo.getLastName());
    user.setEmailAddress(xo.getEmail());
    securitySystem.updateUser(user);
  }

  @PUT
  @Path("/{userId}/password")
  @RequiresUser
  @RequiresAuthentication
  @RequiresPermissions("astrum:userschangepw:create")
  @Validate
  public void changePassword(
      @PathParam("userId") @NotNull final String userId,
      @NotNull @Valid final UserAccountPasswordXO xo) throws Exception
  {
    if (authTickets.redeemTicket(xo.getAuthToken())) {
      if (isAnonymousUser(userId)) {
        throw new WebApplicationMessageException(Status.BAD_REQUEST,
            "Password cannot be changed for user " + userId + ", as it is configured as the Anonymous user");
      }
      securitySystem.changePassword(userId, xo.getPassword());
    }
    else {
      throw new WebApplicationMessageException(Status.FORBIDDEN, "Invalid authentication ticket");
    }
  }

  private User getCurrentUser() {
    try {
      User user = securitySystem.currentUser();
      if (user == null) {
        throw new WebApplicationMessageException(Status.BAD_REQUEST, "Unable to get current user");
      }
      return user;
    }
    catch (UserNotFoundException e) {
      throw new WebApplicationMessageException(Status.BAD_REQUEST, "Unable to get current user");
    }
  }

  UserAccountXO convert(final User user) {
    UserAccountXO xo = new UserAccountXO();
    xo.setUserId(user.getUserId());
    xo.setFirstName(user.getFirstName());
    xo.setLastName(user.getLastName());
    xo.setEmail(user.getEmailAddress());
    xo.setExternal(!DEFAULT_SOURCE.equals(user.getSource()));
    return xo;
  }

  private boolean isAnonymousUser(final String userId) {
    AnonymousConfiguration config = anonymousManager.getConfiguration();
    return config.isEnabled() && config.getUserId().equals(userId);
  }
}
