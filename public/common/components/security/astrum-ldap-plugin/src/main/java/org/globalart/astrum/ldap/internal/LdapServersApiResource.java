package org.globalart.astrum.ldap.internal;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

import jakarta.ws.rs.Consumes;
import jakarta.ws.rs.DELETE;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.POST;
import jakarta.ws.rs.PUT;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.PathParam;
import jakarta.ws.rs.Produces;
import jakarta.ws.rs.QueryParam;
import jakarta.ws.rs.core.Response;
import jakarta.ws.rs.core.Response.Status;

import org.globalart.astrum.rest.Resource;
import org.globalart.astrum.rest.ValidationErrorsException;
import org.globalart.astrum.rest.WebApplicationMessageException;
import org.globalart.astrum.security.internal.rest.SecurityApiConstants;

import org.apache.shiro.authz.annotation.Logical;
import org.apache.shiro.authz.annotation.RequiresAuthentication;
import org.apache.shiro.authz.annotation.RequiresPermissions;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.base.Strings.isNullOrEmpty;
import static jakarta.ws.rs.core.MediaType.APPLICATION_JSON;

/**
 * REST API for LDAP servers. Servers are addressed by name; the list is returned in the order servers are tried.
 */
@Component
@Path(LdapServersApiResource.RESOURCE_URI)
@Consumes(APPLICATION_JSON)
@Produces(APPLICATION_JSON)
public class LdapServersApiResource
    implements Resource
{
  static final String RESOURCE_URI = SecurityApiConstants.V1_RESOURCE_URI + "ldap";

  private final LdapConfigurationManager configuration;

  private final LdapManager ldapManager;

  @Autowired
  public LdapServersApiResource(final LdapConfigurationManager configuration, final LdapManager ldapManager) {
    this.configuration = checkNotNull(configuration);
    this.ldapManager = checkNotNull(ldapManager);
  }

  @GET
  @RequiresAuthentication
  @RequiresPermissions("astrum:ldap:read")
  public List<LdapServerApiXO> list() {
    List<LdapServer> servers = configuration.list();
    List<LdapServerApiXO> result = new ArrayList<>();
    for (int i = 0; i < servers.size(); i++) {
      result.add(LdapServerConverter.toApi(servers.get(i), i + 1));
    }
    return result;
  }

  @GET
  @Path("templates")
  @RequiresAuthentication
  @RequiresPermissions("astrum:ldap:read")
  public List<LdapSchemaTemplateXO> templates() {
    return LdapSchemaTemplates.all();
  }

  @GET
  @Path("{name}")
  @RequiresAuthentication
  @RequiresPermissions("astrum:ldap:read")
  public LdapServerApiXO get(@PathParam("name") final String name) {
    List<LdapServer> servers = configuration.list();
    LdapServer server = configuration.findByName(name).orElseThrow(() -> notFound(name));
    for (int i = 0; i < servers.size(); i++) {
      if (servers.get(i).getId().equals(server.getId())) {
        return LdapServerConverter.toApi(server, i + 1);
      }
    }
    throw notFound(name);
  }

  @POST
  @RequiresAuthentication
  @RequiresPermissions("astrum:ldap:create")
  public Response create(final LdapServerApiXO xo) {
    configuration.create(LdapServerConverter.fromApi(required(xo)), xo.getAuthPassword());
    return Response.status(Status.CREATED).build();
  }

  /**
   * The server is found by the {@code id} in the body when there is one, so the name can be changed.
   */
  @PUT
  @Path("{name}")
  @RequiresAuthentication
  @RequiresPermissions("astrum:ldap:update")
  public void update(@PathParam("name") final String name, final LdapServerApiXO xo) {
    LdapServer existing = existing(required(xo).getId(), name).orElseThrow(() -> notFound(name));
    try {
      configuration.update(existing.getId(), LdapServerConverter.fromApi(xo), xo.getAuthPassword());
    }
    catch (LdapServerNotFoundException e) {
      throw notFound(name);
    }
  }

  @DELETE
  @Path("{name}")
  @RequiresAuthentication
  @RequiresPermissions("astrum:ldap:delete")
  public void delete(@PathParam("name") final String name) {
    LdapServer existing = configuration.findByName(name).orElseThrow(() -> notFound(name));
    try {
      configuration.delete(existing.getId());
    }
    catch (LdapServerNotFoundException e) {
      throw notFound(name);
    }
  }

  /**
   * @param names server names in the order they should be tried
   */
  @POST
  @Path("change-order")
  @RequiresAuthentication
  @RequiresPermissions("astrum:ldap:update")
  public void changeOrder(final List<String> names) {
    if (names == null) {
      throw new ValidationErrorsException("The server order is missing");
    }
    ValidationErrorsException errors = new ValidationErrorsException();
    List<String> ids = new ArrayList<>();
    for (String name : names) {
      configuration.findByName(name).ifPresentOrElse(
          server -> ids.add(server.getId()),
          () -> errors.withError("order", "LDAP server '" + name + "' does not exist"));
    }
    if (errors.hasValidationErrors()) {
      throw errors;
    }
    configuration.changeOrder(ids);
  }

  @DELETE
  @Path("cache")
  @RequiresAuthentication
  @RequiresPermissions("astrum:ldap:delete")
  public void clearCache() {
    ldapManager.clearCache();
  }

  /**
   * @param existingServerName the server being edited, whose stored bind password is used when none is given
   */
  @POST
  @Path("verify-connection")
  @RequiresAuthentication
  @RequiresPermissions(value = {"astrum:ldap:create", "astrum:ldap:update"}, logical = Logical.OR)
  public void verifyConnection(
      @QueryParam("existingServerName") final String existingServerName,
      final LdapServerApiXO xo)
  {
    LdapServer server = configuration.validateConnection(LdapServerConverter.fromApi(required(xo)));
    char[] bindPassword = bindPassword(server, xo, existingServerName);
    verifying(() -> ldapManager.verifyConnection(server, bindPassword));
  }

  @POST
  @Path("verify-user-mapping")
  @RequiresAuthentication
  @RequiresPermissions(value = {"astrum:ldap:create", "astrum:ldap:update"}, logical = Logical.OR)
  public List<LdapUserXO> verifyUserMapping(
      @QueryParam("existingServerName") final String existingServerName,
      final LdapServerApiXO xo)
  {
    LdapServer server = configuration.validateAll(LdapServerConverter.fromApi(required(xo)));
    char[] bindPassword = bindPassword(server, xo, existingServerName);
    List<LdapUserXO> users = new ArrayList<>();
    verifying(() -> ldapManager.verifyUserMapping(server, bindPassword).stream()
        .map(LdapServerConverter::toUserXO)
        .forEach(users::add));
    return users;
  }

  @POST
  @Path("verify-login")
  @RequiresAuthentication
  @RequiresPermissions(value = {"astrum:ldap:create", "astrum:ldap:update"}, logical = Logical.OR)
  public void verifyLogin(
      @QueryParam("existingServerName") final String existingServerName,
      final LdapVerifyLoginXO body)
  {
    if (body == null || body.getLdapServer() == null) {
      throw new ValidationErrorsException("ldapServer", "The LDAP server is missing");
    }
    LdapServerApiXO xo = body.getLdapServer();
    LdapServer server = configuration.validateAll(LdapServerConverter.fromApi(xo));
    String username = LdapServerConverter.decodeBase64("base64Username", body.getBase64Username());
    String password = LdapServerConverter.decodeBase64("base64Password", body.getBase64Password());
    if (isNullOrEmpty(username) || isNullOrEmpty(password)) {
      throw new ValidationErrorsException("Username and password are required");
    }
    char[] bindPassword = bindPassword(server, xo, existingServerName);
    verifying(() -> ldapManager.verifyLogin(server, bindPassword, username, password.toCharArray()));
  }

  private char[] bindPassword(final LdapServer server, final LdapServerApiXO xo, final String existingServerName) {
    return configuration.bindPassword(server, xo.getAuthPassword(), existing(xo.getId(), existingServerName));
  }

  private Optional<LdapServer> existing(final String id, final String name) {
    Optional<LdapServer> byId = configuration.findById(id);
    return byId.isPresent() ? byId : configuration.findByName(name);
  }

  /**
   * Reports a server that cannot be reached, or rejects the settings, as a validation error.
   */
  private static void verifying(final Runnable verification) {
    try {
      verification.run();
    }
    catch (LdapServerException e) {
      throw new ValidationErrorsException("*", e.getMessage());
    }
  }

  private static LdapServerApiXO required(final LdapServerApiXO xo) {
    if (xo == null) {
      throw new ValidationErrorsException("The LDAP server is missing");
    }
    return xo;
  }

  private static WebApplicationMessageException notFound(final String name) {
    return new WebApplicationMessageException(Status.NOT_FOUND, "LDAP server '" + name + "' does not exist",
        APPLICATION_JSON);
  }
}
