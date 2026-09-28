package org.globalart.astrum.oidc.internal;

import java.util.LinkedHashMap;
import java.util.Map;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;

@JsonIgnoreProperties(ignoreUnknown = true)
public class OidcConfiguration
{
  public static final String DEFAULT_USERNAME_CLAIM = "preferred_username";

  public static final String DEFAULT_FIRST_NAME_CLAIM = "given_name";

  public static final String DEFAULT_LAST_NAME_CLAIM = "family_name";

  public static final String DEFAULT_EMAIL_CLAIM = "email";

  private String issuerUrl;

  private String clientId;

  private String clientSecretId;

  private String idpAuthorizationUrl;

  private String idpTokenUrl;

  private String idpJwksUrl;

  private String idpLogoutUrl;

  private String idpJwsAlgorithm;

  private String idpJwks;

  private String usernameClaim;

  private String firstNameClaim;

  private String lastNameClaim;

  private String emailClaim;

  private String groupsClaim;

  private Map<String, String> exactMatchClaims = new LinkedHashMap<>();

  private Map<String, String> authorizationCustomParams = new LinkedHashMap<>();

  private Map<String, String> tokenRequestCustomParams = new LinkedHashMap<>();

  private boolean useTrustStore;

  public String getIssuerUrl() {
    return issuerUrl;
  }

  public void setIssuerUrl(final String issuerUrl) {
    this.issuerUrl = issuerUrl;
  }

  public String getClientId() {
    return clientId;
  }

  public void setClientId(final String clientId) {
    this.clientId = clientId;
  }

  public String getClientSecretId() {
    return clientSecretId;
  }

  public void setClientSecretId(final String clientSecretId) {
    this.clientSecretId = clientSecretId;
  }

  public String getIdpAuthorizationUrl() {
    return idpAuthorizationUrl;
  }

  public void setIdpAuthorizationUrl(final String idpAuthorizationUrl) {
    this.idpAuthorizationUrl = idpAuthorizationUrl;
  }

  public String getIdpTokenUrl() {
    return idpTokenUrl;
  }

  public void setIdpTokenUrl(final String idpTokenUrl) {
    this.idpTokenUrl = idpTokenUrl;
  }

  public String getIdpJwksUrl() {
    return idpJwksUrl;
  }

  public void setIdpJwksUrl(final String idpJwksUrl) {
    this.idpJwksUrl = idpJwksUrl;
  }

  public String getIdpLogoutUrl() {
    return idpLogoutUrl;
  }

  public void setIdpLogoutUrl(final String idpLogoutUrl) {
    this.idpLogoutUrl = idpLogoutUrl;
  }

  public String getIdpJwsAlgorithm() {
    return idpJwsAlgorithm;
  }

  public void setIdpJwsAlgorithm(final String idpJwsAlgorithm) {
    this.idpJwsAlgorithm = idpJwsAlgorithm;
  }

  public String getIdpJwks() {
    return idpJwks;
  }

  public void setIdpJwks(final String idpJwks) {
    this.idpJwks = idpJwks;
  }

  public String getUsernameClaim() {
    return usernameClaim;
  }

  public void setUsernameClaim(final String usernameClaim) {
    this.usernameClaim = usernameClaim;
  }

  public String getFirstNameClaim() {
    return firstNameClaim;
  }

  public void setFirstNameClaim(final String firstNameClaim) {
    this.firstNameClaim = firstNameClaim;
  }

  public String getLastNameClaim() {
    return lastNameClaim;
  }

  public void setLastNameClaim(final String lastNameClaim) {
    this.lastNameClaim = lastNameClaim;
  }

  public String getEmailClaim() {
    return emailClaim;
  }

  public void setEmailClaim(final String emailClaim) {
    this.emailClaim = emailClaim;
  }

  public String getGroupsClaim() {
    return groupsClaim;
  }

  public void setGroupsClaim(final String groupsClaim) {
    this.groupsClaim = groupsClaim;
  }

  public Map<String, String> getExactMatchClaims() {
    return exactMatchClaims;
  }

  public void setExactMatchClaims(final Map<String, String> exactMatchClaims) {
    this.exactMatchClaims = exactMatchClaims == null ? new LinkedHashMap<>() : exactMatchClaims;
  }

  public Map<String, String> getAuthorizationCustomParams() {
    return authorizationCustomParams;
  }

  public void setAuthorizationCustomParams(final Map<String, String> authorizationCustomParams) {
    this.authorizationCustomParams = authorizationCustomParams == null ? new LinkedHashMap<>() : authorizationCustomParams;
  }

  public Map<String, String> getTokenRequestCustomParams() {
    return tokenRequestCustomParams;
  }

  public void setTokenRequestCustomParams(final Map<String, String> tokenRequestCustomParams) {
    this.tokenRequestCustomParams = tokenRequestCustomParams == null ? new LinkedHashMap<>() : tokenRequestCustomParams;
  }

  public boolean isUseTrustStore() {
    return useTrustStore;
  }

  public void setUseTrustStore(final boolean useTrustStore) {
    this.useTrustStore = useTrustStore;
  }
}
