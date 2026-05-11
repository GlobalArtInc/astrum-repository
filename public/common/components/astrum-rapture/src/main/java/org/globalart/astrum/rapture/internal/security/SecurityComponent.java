
package org.globalart.astrum.rapture.internal.security;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.validation.constraints.NotEmpty;

import org.globalart.astrum.common.text.Strings2;
import org.globalart.astrum.common.wonderland.AuthTicketService;
import org.globalart.astrum.extdirect.DirectComponentSupport;
import org.globalart.astrum.rapture.StateContributor;
import org.globalart.astrum.security.SecuritySystem;
import org.globalart.astrum.security.anonymous.AnonymousConfiguration;
import org.globalart.astrum.security.anonymous.AnonymousHelper;
import org.globalart.astrum.security.anonymous.AnonymousManager;
import org.globalart.astrum.security.authz.WildcardPermission2;
import org.globalart.astrum.security.privilege.Privilege;
import org.globalart.astrum.validation.Validate;

import com.codahale.metrics.annotation.ExceptionMetered;
import com.codahale.metrics.annotation.Timed;
import com.softwarementors.extjs.djn.config.annotations.DirectAction;
import com.softwarementors.extjs.djn.config.annotations.DirectMethod;
import org.apache.shiro.SecurityUtils;
import org.apache.shiro.authc.AuthenticationException;
import org.apache.shiro.authc.UsernamePasswordToken;
import org.apache.shiro.authz.Permission;
import org.apache.shiro.subject.Subject;

import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.base.Preconditions.checkState;
import org.springframework.stereotype.Component;

/**
 * Security Ext.Direct component.
 *
 * @since 3.0
 */
@Component
@DirectAction(action = "rapture_Security")
public class SecurityComponent
    extends DirectComponentSupport
    implements StateContributor
{
  private final SecuritySystem securitySystem;

  private final AnonymousManager anonymousManager;

  private final AuthTicketService authTickets;

  @Autowired
  public SecurityComponent(
      final SecuritySystem securitySystem,
      final AnonymousManager anonymousManager,
      final AuthTicketService authTickets)
  {
    this.securitySystem = checkNotNull(securitySystem);
    this.anonymousManager = checkNotNull(anonymousManager);
    this.authTickets = checkNotNull(authTickets);
  }

  // FIXME: Move authenticate to session servlet

  @DirectMethod
  @Timed
  @ExceptionMetered
  @Validate
  public UserXO authenticate(
      @NotEmpty final String base64Username,
      @NotEmpty final String base64Password) throws Exception
  {
    Subject subject = securitySystem.getSubject();

    // FIXME: Subject is not nullable, but we have code that checks for nulls, likely from testing setups, verify and
    // simplify
    checkState(subject != null);

    try {
      subject.login(new UsernamePasswordToken(
          Strings2.decodeBase64(base64Username),
          Strings2.decodeBase64(base64Password),
          false));
    }
    catch (Exception e) {
      throw new Exception("Authentication failed", e);
    }

    return getUser();
  }

  @DirectMethod
  @Timed
  @ExceptionMetered
  @Validate
  public String authenticationToken(
      @NotEmpty final String base64Username,
      @NotEmpty final String base64Password) throws Exception
  {
    Subject subject = securitySystem.getSubject();
    if (subject == null || !subject.isAuthenticated()) {
      authenticate(base64Username, base64Password);
    }

    String username = Strings2.decodeBase64(base64Username);
    String password = Strings2.decodeBase64(base64Password);
    log.debug("Authenticate w/username: {}, password: {}", username, Strings2.mask(password));

    // Require current user to be the requested user to authenticate
    subject = securitySystem.getSubject();
    if (!subject.getPrincipal().toString().equals(username)) {
      throw new Exception("Username mismatch");
    }

    // Ask the sec-manager to authenticate, this won't alter the current subject
    try {
      SecurityUtils.getSecurityManager().authenticate(new UsernamePasswordToken(username, password));
    }
    catch (AuthenticationException e) {
      throw new Exception("Authentication failed", e);
    }

    // At this point we should be authenticated, return a new ticket
    Optional<String> realmName = subject.getPrincipals().getRealmNames().stream().findFirst();
    return authTickets.createTicket(username, realmName.orElse(null));
  }

  @DirectMethod
  @Timed
  @ExceptionMetered
  public UserXO getUser() {
    UserXO userXO = null;

    Subject subject = securitySystem.getSubject();
    // Anonymous subjects must NOT be reported here even after ASTRUM-47113: the ExtJS UI uses
    // getUser() == null as the signal to show the Sign In button. Returning a populated UserXO
    // for anonymous would hide the button and strand the UI in an anonymous session after logout.
    // See getPermissions() below for the anonymous permission path.
    if (subject != null && subject.isAuthenticated()) {
      userXO = new UserXO();
      userXO.setAuthenticated(subject.isAuthenticated());
      userXO.setAuthenticatedRealms(subject.getPrincipals().getRealmNames());

      // HACK: roles for the current user are not exposed to the UI.
      // HACK: but we need to know if user is admin or not for some things (like outreach)
      if (subject.isPermitted(new WildcardPermission2("astrum:*"))) {
        userXO.setAdministrator(true);
      }

      Object principal = subject.getPrincipal();
      if (principal != null) {
        userXO.setId(principal.toString());
      }
    }
    return userXO;
  }

  @DirectMethod
  @Timed
  @ExceptionMetered
  public List<PermissionXO> getPermissions() {
    List<PermissionXO> permissions = null;
    Subject subject = securitySystem.getSubject();
    if (subject != null
        && (subject.isAuthenticated() || subject.isRemembered() || AnonymousHelper.isAnonymous(subject))) {
      permissions = calculatePermissions(subject);
    }
    return permissions;
  }

  @Override
  public Map<String, Object> getState() {
    Map<String, Object> state = new HashMap<>();
    state.put("user", getUser());

    AnonymousConfiguration anonymousConfiguration = anonymousManager.getConfiguration();
    state.put("anonymousUsername", anonymousConfiguration.isEnabled() ? anonymousConfiguration.getUserId() : null);
    return state;
  }

  private List<PermissionXO> calculatePermissions(final Subject subject) {
    log.debug("Calculating permissions");

    List<Permission> granted = new ArrayList<>();
    List<PermissionXO> result = new ArrayList<>();

    // find all privileges which we expose the UI, , which we can deconstruct and evaluate
    for (Privilege privilege : securitySystem.listPrivileges()) {
      // only WildcardPermission2 presently is supported due to toString() implementation
      if (privilege.getPermission() instanceof WildcardPermission2) {
        granted.add(privilege.getPermission());
      }
    }

    // determine which of the exposed privilege permissions the current subject is granted
    boolean[] boolResults = subject.isPermitted(granted);
    for (int i = 0; i < granted.size(); i++) {
      if (boolResults[i]) {
        PermissionXO entry = new PermissionXO();
        entry.setId(granted.get(i).toString());
        result.add(entry);
      }
    }

    return result;
  }
}
