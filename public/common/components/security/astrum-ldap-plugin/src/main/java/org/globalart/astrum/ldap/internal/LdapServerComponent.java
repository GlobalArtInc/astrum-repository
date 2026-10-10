package org.globalart.astrum.ldap.internal;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

import org.globalart.astrum.extdirect.DirectComponent;
import org.globalart.astrum.extdirect.DirectComponentSupport;
import org.globalart.astrum.rest.ValidationErrorsException;

import com.codahale.metrics.annotation.ExceptionMetered;
import com.codahale.metrics.annotation.Timed;
import com.softwarementors.extjs.djn.config.annotations.DirectAction;
import com.softwarementors.extjs.djn.config.annotations.DirectMethod;
import org.apache.shiro.authz.annotation.Logical;
import org.apache.shiro.authz.annotation.RequiresAuthentication;
import org.apache.shiro.authz.annotation.RequiresPermissions;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.base.Strings.isNullOrEmpty;

/**
 * LDAP servers {@link DirectComponent} for the classic UI.
 */
@Component
@DirectAction(action = "ldap_LdapServer")
public class LdapServerComponent
    extends DirectComponentSupport
{
  private final LdapConfigurationManager configuration;

  private final LdapManager ldapManager;

  @Autowired
  public LdapServerComponent(final LdapConfigurationManager configuration, final LdapManager ldapManager) {
    this.configuration = checkNotNull(configuration);
    this.ldapManager = checkNotNull(ldapManager);
  }

  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresPermissions("astrum:ldap:read")
  public List<LdapServerXO> read() {
    List<LdapServer> servers = configuration.list();
    List<LdapServerXO> result = new ArrayList<>();
    for (int i = 0; i < servers.size(); i++) {
      result.add(LdapServerConverter.toExtDirect(servers.get(i), i + 1));
    }
    return result;
  }

  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresPermissions("astrum:ldap:read")
  public List<LdapSchemaTemplateXO> readTemplates() {
    return LdapSchemaTemplates.all();
  }

  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresAuthentication
  @RequiresPermissions("astrum:ldap:create")
  public LdapServerXO create(final LdapServerXO xo) {
    LdapServer created = configuration.create(LdapServerConverter.fromExtDirect(required(xo)), xo.getAuthPassword());
    return LdapServerConverter.toExtDirect(created, configuration.list().size());
  }

  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresAuthentication
  @RequiresPermissions("astrum:ldap:update")
  public LdapServerXO update(final LdapServerXO xo) {
    if (isNullOrEmpty(required(xo).getId())) {
      throw new ValidationErrorsException("id", "The LDAP server id is required");
    }
    LdapServer updated = configuration.update(xo.getId(), LdapServerConverter.fromExtDirect(xo), xo.getAuthPassword());
    return LdapServerConverter.toExtDirect(updated, orderOf(updated.getId()));
  }

  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresAuthentication
  @RequiresPermissions("astrum:ldap:delete")
  public void remove(final String id) {
    configuration.delete(id);
  }

  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresAuthentication
  @RequiresPermissions("astrum:ldap:update")
  public void changeOrder(final List<String> ids) {
    configuration.changeOrder(ids);
  }

  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresAuthentication
  @RequiresPermissions("astrum:ldap:delete")
  public void clearCache() {
    ldapManager.clearCache();
  }

  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresAuthentication
  @RequiresPermissions(value = {"astrum:ldap:create", "astrum:ldap:update"}, logical = Logical.OR)
  public void verifyConnection(final LdapServerXO xo) {
    LdapServer server = configuration.validateConnection(LdapServerConverter.fromExtDirect(required(xo)));
    char[] bindPassword = bindPassword(server, xo);
    verifying(() -> ldapManager.verifyConnection(server, bindPassword));
  }

  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresAuthentication
  @RequiresPermissions(value = {"astrum:ldap:create", "astrum:ldap:update"}, logical = Logical.OR)
  public List<LdapUserXO> verifyUserMapping(final LdapServerXO xo) {
    LdapServer server = configuration.validateAll(LdapServerConverter.fromExtDirect(required(xo)));
    char[] bindPassword = bindPassword(server, xo);
    List<LdapUserXO> users = new ArrayList<>();
    verifying(() -> ldapManager.verifyUserMapping(server, bindPassword).stream()
        .map(LdapServerConverter::toUserXO)
        .forEach(users::add));
    return users;
  }

  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresAuthentication
  @RequiresPermissions(value = {"astrum:ldap:create", "astrum:ldap:update"}, logical = Logical.OR)
  public void verifyLogin(final LdapServerXO xo, final String base64Username, final String base64Password) {
    LdapServer server = configuration.validateAll(LdapServerConverter.fromExtDirect(required(xo)));
    String username = LdapServerConverter.decodeBase64("username", base64Username);
    String password = LdapServerConverter.decodeBase64("password", base64Password);
    if (isNullOrEmpty(username) || isNullOrEmpty(password)) {
      throw new ValidationErrorsException("Username and password are required");
    }
    char[] bindPassword = bindPassword(server, xo);
    verifying(() -> ldapManager.verifyLogin(server, bindPassword, username, password.toCharArray()));
  }

  /**
   * A failed check is an expected answer, not a server error: report it like a validation error, which is shown to
   * the user without logging a stack trace.
   */
  private static void verifying(final Runnable verification) {
    try {
      verification.run();
    }
    catch (LdapServerException e) {
      throw new ValidationErrorsException(e.getMessage());
    }
  }

  private char[] bindPassword(final LdapServer server, final LdapServerXO xo) {
    Optional<LdapServer> existing = configuration.findById(xo.getId());
    return configuration.bindPassword(server, xo.getAuthPassword(), existing);
  }

  private int orderOf(final String id) {
    List<LdapServer> servers = configuration.list();
    for (int i = 0; i < servers.size(); i++) {
      if (servers.get(i).getId().equals(id)) {
        return i + 1;
      }
    }
    return servers.size();
  }

  private static LdapServerXO required(final LdapServerXO xo) {
    if (xo == null) {
      throw new ValidationErrorsException("The LDAP server is missing");
    }
    return xo;
  }
}
