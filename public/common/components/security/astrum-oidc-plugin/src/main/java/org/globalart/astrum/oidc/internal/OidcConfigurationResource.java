package org.globalart.astrum.oidc.internal;

import java.io.IOException;
import java.net.URI;
import java.net.URISyntaxException;
import java.util.LinkedHashMap;
import java.util.Map;
import java.util.TreeSet;

import jakarta.servlet.http.HttpServletRequest;
import jakarta.ws.rs.Consumes;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.PUT;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.Produces;
import jakarta.ws.rs.core.Context;
import jakarta.ws.rs.core.Response.Status;

import com.fasterxml.jackson.databind.ObjectMapper;
import org.globalart.astrum.rapture.PasswordPlaceholder;
import org.globalart.astrum.rest.Resource;
import org.globalart.astrum.rest.WebApplicationMessageException;
import org.globalart.astrum.security.realm.RealmManager;
import org.apache.shiro.SecurityUtils;
import org.apache.shiro.authz.annotation.RequiresAuthentication;
import org.apache.shiro.authz.annotation.RequiresPermissions;
import org.apache.shiro.subject.Subject;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.base.Strings.isNullOrEmpty;
import static jakarta.ws.rs.core.MediaType.APPLICATION_JSON;

@Component
@Consumes(APPLICATION_JSON)
@Produces(APPLICATION_JSON)
@Path(OidcConfigurationResource.RESOURCE_PATH)
public class OidcConfigurationResource
    implements Resource
{
  static final String RESOURCE_PATH = "internal/ui/oauth2";

  private final OidcConfigurationStore store;

  private final OidcProvider provider;

  private final RealmManager realmManager;

  private final ObjectMapper mapper = new ObjectMapper();

  @Autowired
  public OidcConfigurationResource(
      final OidcConfigurationStore store,
      final OidcProvider provider,
      final RealmManager realmManager)
  {
    this.store = checkNotNull(store);
    this.provider = checkNotNull(provider);
    this.realmManager = checkNotNull(realmManager);
  }

  @GET
  @RequiresAuthentication
  @RequiresPermissions("astrum:settings:read")
  public OidcConfigurationXO read(@Context final HttpServletRequest request) {
    OidcConfigurationXO xo = store.load().map(OidcConfigurationResource::toXO).orElseGet(() -> {
      OidcConfigurationXO empty = new OidcConfigurationXO();
      empty.setUsernameClaim(OidcConfiguration.DEFAULT_USERNAME_CLAIM);
      empty.setFirstNameClaim(OidcConfiguration.DEFAULT_FIRST_NAME_CLAIM);
      empty.setLastNameClaim(OidcConfiguration.DEFAULT_LAST_NAME_CLAIM);
      empty.setEmailClaim(OidcConfiguration.DEFAULT_EMAIL_CLAIM);
      return empty;
    });
    if (request != null) {
      xo.setRedirectUri(OidcServlet.baseUrl(request) + OidcServlet.CALLBACK_PATH);
    }
    return xo;
  }

  @PUT
  @RequiresAuthentication
  @RequiresPermissions("astrum:settings:update")
  public void update(final OidcConfigurationXO xo) {
    if (xo == null) {
      throw badRequest("The configuration is missing");
    }
    OidcConfiguration configuration = fromXO(xo);
    validate(configuration);

    String secret = xo.getClientSecret();
    OidcConfigurationStore.SecretChange secretChange;
    if (PasswordPlaceholder.is(secret)) {
      secretChange = OidcConfigurationStore.SecretChange.keep();
    }
    else if (isNullOrEmpty(secret)) {
      secretChange = OidcConfigurationStore.SecretChange.remove();
    }
    else {
      secretChange = OidcConfigurationStore.SecretChange.set(secret);
    }

    try {
      provider.resolveMetadata(configuration);
    }
    catch (OidcException e) {
      throw badRequest("Could not load the identity provider configuration: " + e.getMessage());
    }

    Subject subject = SecurityUtils.getSubject();
    String userId = subject.getPrincipal() != null ? subject.getPrincipal().toString() : null;
    store.save(configuration, secretChange, userId);
    provider.reset();
    if (!realmManager.isRealmEnabled(OidcRealm.NAME)) {
      realmManager.enableRealm(OidcRealm.NAME);
    }
  }

  private void validate(final OidcConfiguration configuration) {
    if (isNullOrEmpty(configuration.getClientId())) {
      throw badRequest("Client ID is required");
    }
    requireUrl("Issuer URL", configuration.getIssuerUrl());
    requireUrl("Authorization URL", configuration.getIdpAuthorizationUrl());
    requireUrl("Token URL", configuration.getIdpTokenUrl());
    requireUrl("JSON Web Key URL", configuration.getIdpJwksUrl());
    requireUrl("Logout URL", configuration.getIdpLogoutUrl());
    if (isNullOrEmpty(configuration.getIssuerUrl())
        && (isNullOrEmpty(configuration.getIdpAuthorizationUrl()) || isNullOrEmpty(configuration.getIdpTokenUrl())
        || (isNullOrEmpty(configuration.getIdpJwksUrl()) && isNullOrEmpty(configuration.getIdpJwks())))) {
      throw badRequest("Set the Issuer URL, or the Authorization URL, Token URL and JSON Web Keys");
    }
    if (!isNullOrEmpty(configuration.getIdpJwks())) {
      try {
        mapper.readTree(configuration.getIdpJwks());
      }
      catch (IOException e) {
        throw badRequest("JSON Web Keys must be valid JSON");
      }
    }
    if (!isNullOrEmpty(configuration.getIdpJwsAlgorithm())
        && !OidcProvider.SUPPORTED_ALGORITHMS.contains(configuration.getIdpJwsAlgorithm())) {
      throw badRequest("JWT Signature Algorithm must be one of " + String.join(", ",
          new TreeSet<>(OidcProvider.SUPPORTED_ALGORITHMS)));
    }
  }

  private static void requireUrl(final String label, final String value) {
    if (isNullOrEmpty(value)) {
      return;
    }
    try {
      URI uri = new URI(value);
      if (!"https".equalsIgnoreCase(uri.getScheme()) && !"http".equalsIgnoreCase(uri.getScheme())
          || isNullOrEmpty(uri.getHost())) {
        throw badRequest(label + " must be an http or https URL");
      }
    }
    catch (URISyntaxException e) {
      throw badRequest(label + " is not a valid URL");
    }
  }

  private static WebApplicationMessageException badRequest(final String message) {
    return new WebApplicationMessageException(Status.BAD_REQUEST, message);
  }

  private static OidcConfiguration fromXO(final OidcConfigurationXO xo) {
    OidcConfiguration configuration = new OidcConfiguration();
    configuration.setIssuerUrl(trim(xo.getIssuerUrl()));
    configuration.setClientId(trim(xo.getClientId()));
    configuration.setIdpAuthorizationUrl(trim(xo.getIdpAuthorizationUrl()));
    configuration.setIdpTokenUrl(trim(xo.getIdpTokenUrl()));
    configuration.setIdpJwksUrl(trim(xo.getIdpJwksUrl()));
    configuration.setIdpLogoutUrl(trim(xo.getIdpLogoutUrl()));
    configuration.setIdpJwsAlgorithm(trim(xo.getIdpJwsAlgorithm()));
    configuration.setIdpJwks(trim(xo.getIdpJwks()));
    configuration.setUsernameClaim(trim(xo.getUsernameClaim()));
    configuration.setFirstNameClaim(trim(xo.getFirstNameClaim()));
    configuration.setLastNameClaim(trim(xo.getLastNameClaim()));
    configuration.setEmailClaim(trim(xo.getEmailClaim()));
    configuration.setGroupsClaim(trim(xo.getGroupsClaim()));
    configuration.setExactMatchClaims(copy(xo.getExactMatchClaims()));
    configuration.setAuthorizationCustomParams(copy(xo.getAuthorizationCustomParams()));
    configuration.setTokenRequestCustomParams(copy(xo.getTokenRequestCustomParams()));
    configuration.setUseTrustStore(xo.isUseTrustStore());
    return configuration;
  }

  private static OidcConfigurationXO toXO(final OidcConfiguration configuration) {
    OidcConfigurationXO xo = new OidcConfigurationXO();
    xo.setIssuerUrl(configuration.getIssuerUrl());
    xo.setClientId(configuration.getClientId());
    xo.setClientSecret(configuration.getClientSecretId() != null ? PasswordPlaceholder.get() : "");
    xo.setIdpAuthorizationUrl(configuration.getIdpAuthorizationUrl());
    xo.setIdpTokenUrl(configuration.getIdpTokenUrl());
    xo.setIdpJwksUrl(configuration.getIdpJwksUrl());
    xo.setIdpLogoutUrl(configuration.getIdpLogoutUrl());
    xo.setIdpJwsAlgorithm(configuration.getIdpJwsAlgorithm());
    xo.setIdpJwks(configuration.getIdpJwks());
    xo.setUsernameClaim(configuration.getUsernameClaim());
    xo.setFirstNameClaim(configuration.getFirstNameClaim());
    xo.setLastNameClaim(configuration.getLastNameClaim());
    xo.setEmailClaim(configuration.getEmailClaim());
    xo.setGroupsClaim(configuration.getGroupsClaim());
    xo.setExactMatchClaims(copy(configuration.getExactMatchClaims()));
    xo.setAuthorizationCustomParams(copy(configuration.getAuthorizationCustomParams()));
    xo.setTokenRequestCustomParams(copy(configuration.getTokenRequestCustomParams()));
    xo.setUseTrustStore(configuration.isUseTrustStore());
    return xo;
  }

  private static Map<String, String> copy(final Map<String, String> source) {
    Map<String, String> result = new LinkedHashMap<>();
    if (source != null) {
      source.forEach((key, value) -> {
        if (!isNullOrEmpty(key) && value != null) {
          result.put(key.trim(), value);
        }
      });
    }
    return result;
  }

  private static String trim(final String value) {
    if (value == null) {
      return null;
    }
    String trimmed = value.trim();
    return trimmed.isEmpty() ? null : trimmed;
  }
}
