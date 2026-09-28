package org.globalart.astrum.oidc.internal;

import java.io.IOException;
import java.math.BigInteger;
import java.net.URLEncoder;
import java.nio.charset.StandardCharsets;
import java.security.AlgorithmParameters;
import java.security.GeneralSecurityException;
import java.security.KeyFactory;
import java.security.PublicKey;
import java.security.interfaces.ECPublicKey;
import java.security.interfaces.RSAPublicKey;
import java.security.spec.ECGenParameterSpec;
import java.security.spec.ECParameterSpec;
import java.security.spec.ECPoint;
import java.security.spec.ECPublicKeySpec;
import java.security.spec.RSAPublicKeySpec;
import java.util.ArrayList;
import java.util.Base64;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Set;

import jakarta.inject.Provider;

import com.auth0.jwt.JWT;
import com.auth0.jwt.algorithms.Algorithm;
import com.auth0.jwt.exceptions.JWTVerificationException;
import com.auth0.jwt.interfaces.DecodedJWT;
import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.apache.http.NameValuePair;
import org.apache.http.client.entity.UrlEncodedFormEntity;
import org.apache.http.client.methods.CloseableHttpResponse;
import org.apache.http.client.methods.HttpGet;
import org.apache.http.client.methods.HttpPost;
import org.apache.http.client.methods.HttpUriRequest;
import org.apache.http.impl.client.CloseableHttpClient;
import org.apache.http.impl.client.HttpClients;
import org.apache.http.message.BasicNameValuePair;
import org.apache.http.util.EntityUtils;
import org.globalart.astrum.httpclient.HttpClientManager;
import org.globalart.astrum.ssl.TrustStore;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.base.Strings.isNullOrEmpty;

@Component
public class OidcProvider
{
  private static final Logger log = LoggerFactory.getLogger(OidcProvider.class);

  static final Set<String> SUPPORTED_ALGORITHMS = Set.of("RS256", "RS384", "RS512", "ES256", "ES384", "ES512");

  private static final long METADATA_TTL_MILLIS = 60 * 60 * 1000L;

  private static final long JWKS_REFRESH_MIN_INTERVAL_MILLIS = 60 * 1000L;

  private static final long CLOCK_SKEW_SECONDS = 60;

  private static final int MAX_RESPONSE_BYTES = 1024 * 1024;

  private final HttpClientManager httpClientManager;

  private final Provider<TrustStore> trustStore;

  private final ObjectMapper mapper = new ObjectMapper();

  private Metadata metadata;

  private String metadataFingerprint;

  private long metadataLoadedAt;

  private Map<String, PublicKey> keys = Map.of();

  private String keysFingerprint;

  private long keysLoadedAt;

  @Autowired
  public OidcProvider(final HttpClientManager httpClientManager, final Provider<TrustStore> trustStore) {
    this.httpClientManager = checkNotNull(httpClientManager);
    this.trustStore = checkNotNull(trustStore);
  }

  public record Metadata(
      String issuer,
      String authorizationEndpoint,
      String tokenEndpoint,
      String userinfoEndpoint,
      String jwksUri,
      String endSessionEndpoint,
      List<String> tokenEndpointAuthMethods)
  {
  }

  public record Tokens(String idToken, String accessToken)
  {
  }

  public synchronized void reset() {
    metadata = null;
    metadataFingerprint = null;
    keys = Map.of();
    keysFingerprint = null;
  }

  public synchronized Metadata metadata(final OidcConfiguration configuration) throws OidcException {
    String fingerprint = fingerprint(configuration);
    if (metadata != null && fingerprint.equals(metadataFingerprint)
        && System.currentTimeMillis() - metadataLoadedAt < METADATA_TTL_MILLIS) {
      return metadata;
    }
    metadata = resolveMetadata(configuration);
    metadataFingerprint = fingerprint;
    metadataLoadedAt = System.currentTimeMillis();
    return metadata;
  }

  Metadata resolveMetadata(final OidcConfiguration configuration) throws OidcException {
    JsonNode discovery = null;
    if (!isNullOrEmpty(configuration.getIssuerUrl())) {
      String issuerUrl = stripTrailingSlash(configuration.getIssuerUrl());
      discovery = getJson(configuration, issuerUrl + "/.well-known/openid-configuration", null);
    }
    String issuer = discovery != null ? text(discovery, "issuer") : null;
    if (isNullOrEmpty(issuer) && !isNullOrEmpty(configuration.getIssuerUrl())) {
      issuer = stripTrailingSlash(configuration.getIssuerUrl());
    }
    List<String> authMethods = new ArrayList<>();
    if (discovery != null && discovery.has("token_endpoint_auth_methods_supported")) {
      discovery.get("token_endpoint_auth_methods_supported").forEach(node -> authMethods.add(node.asText()));
    }
    Metadata resolved = new Metadata(
        issuer,
        firstNonEmpty(configuration.getIdpAuthorizationUrl(), text(discovery, "authorization_endpoint")),
        firstNonEmpty(configuration.getIdpTokenUrl(), text(discovery, "token_endpoint")),
        text(discovery, "userinfo_endpoint"),
        firstNonEmpty(configuration.getIdpJwksUrl(), text(discovery, "jwks_uri")),
        firstNonEmpty(configuration.getIdpLogoutUrl(), text(discovery, "end_session_endpoint")),
        List.copyOf(authMethods));
    if (isNullOrEmpty(resolved.authorizationEndpoint())) {
      throw new OidcException("The provider does not define an authorization endpoint");
    }
    if (isNullOrEmpty(resolved.tokenEndpoint())) {
      throw new OidcException("The provider does not define a token endpoint");
    }
    if (isNullOrEmpty(resolved.jwksUri()) && isNullOrEmpty(configuration.getIdpJwks())) {
      throw new OidcException("The provider does not define a JWKS endpoint and no JSON Web Keys are configured");
    }
    return resolved;
  }

  public Tokens exchangeCode(
      final OidcConfiguration configuration,
      final char[] clientSecret,
      final String code,
      final String redirectUri,
      final String codeVerifier) throws OidcException
  {
    Metadata provider = metadata(configuration);
    List<NameValuePair> form = new ArrayList<>();
    form.add(new BasicNameValuePair("grant_type", "authorization_code"));
    form.add(new BasicNameValuePair("code", code));
    form.add(new BasicNameValuePair("redirect_uri", redirectUri));
    form.add(new BasicNameValuePair("code_verifier", codeVerifier));
    form.add(new BasicNameValuePair("client_id", configuration.getClientId()));
    String authorization = null;
    boolean usePost = !provider.tokenEndpointAuthMethods().isEmpty()
        && !provider.tokenEndpointAuthMethods().contains("client_secret_basic")
        && provider.tokenEndpointAuthMethods().contains("client_secret_post");
    if (clientSecret != null && clientSecret.length > 0) {
      if (usePost) {
        form.add(new BasicNameValuePair("client_secret", new String(clientSecret)));
      }
      else {
        String credentials = formEncode(configuration.getClientId()) + ":" + formEncode(new String(clientSecret));
        authorization = "Basic " + Base64.getEncoder().encodeToString(credentials.getBytes(StandardCharsets.UTF_8));
      }
    }
    configuration.getTokenRequestCustomParams().forEach((name, value) -> form.add(new BasicNameValuePair(name, value)));

    HttpPost post = new HttpPost(provider.tokenEndpoint());
    post.setHeader("Accept", "application/json");
    if (authorization != null) {
      post.setHeader("Authorization", authorization);
    }
    post.setEntity(new UrlEncodedFormEntity(form, StandardCharsets.UTF_8));
    JsonNode response = execute(configuration, post);
    String idToken = text(response, "id_token");
    if (isNullOrEmpty(idToken)) {
      throw new OidcException("The token response does not contain an ID token");
    }
    return new Tokens(idToken, text(response, "access_token"));
  }

  public Map<String, Object> verifyIdToken(
      final OidcConfiguration configuration,
      final String idToken,
      final String nonce) throws OidcException
  {
    Metadata provider = metadata(configuration);
    DecodedJWT unverified;
    try {
      unverified = JWT.decode(idToken);
    }
    catch (JWTVerificationException e) {
      throw new OidcException("The ID token is malformed", e);
    }
    String algorithmName = unverified.getAlgorithm();
    if (!isNullOrEmpty(configuration.getIdpJwsAlgorithm())
        && !configuration.getIdpJwsAlgorithm().trim().equals(algorithmName)) {
      throw new OidcException("The ID token is signed with " + algorithmName + " instead of "
          + configuration.getIdpJwsAlgorithm().trim());
    }
    PublicKey key = key(configuration, provider, unverified.getKeyId());
    Algorithm algorithm = algorithm(algorithmName, key);
    try {
      com.auth0.jwt.interfaces.Verification verification = JWT.require(algorithm)
          .withAudience(configuration.getClientId())
          .withClaim("nonce", nonce)
          .acceptLeeway(CLOCK_SKEW_SECONDS);
      if (!isNullOrEmpty(provider.issuer())) {
        verification = verification.withIssuer(provider.issuer());
      }
      DecodedJWT verified = verification.build().verify(idToken);
      Map<String, Object> claims = mapper.readValue(
          Base64.getUrlDecoder().decode(verified.getPayload()), new TypeReference<Map<String, Object>>() { });
      List<String> audience = verified.getAudience();
      Object authorizedParty = claims.get("azp");
      if (audience != null && audience.size() > 1 && authorizedParty != null
          && !configuration.getClientId().equals(authorizedParty)) {
        throw new OidcException("The ID token was issued to another client");
      }
      return claims;
    }
    catch (JWTVerificationException e) {
      throw new OidcException("The ID token is not valid: " + e.getMessage(), e);
    }
    catch (IOException e) {
      throw new OidcException("The ID token payload is not valid JSON", e);
    }
  }

  public Map<String, Object> userInfo(
      final OidcConfiguration configuration,
      final String accessToken) throws OidcException
  {
    Metadata provider = metadata(configuration);
    if (isNullOrEmpty(provider.userinfoEndpoint()) || isNullOrEmpty(accessToken)) {
      return Map.of();
    }
    JsonNode node = getJson(configuration, provider.userinfoEndpoint(), "Bearer " + accessToken);
    return mapper.convertValue(node, new TypeReference<Map<String, Object>>() { });
  }

  private synchronized PublicKey key(
      final OidcConfiguration configuration,
      final Metadata provider,
      final String keyId) throws OidcException
  {
    String fingerprint = fingerprint(configuration);
    boolean stale = !fingerprint.equals(keysFingerprint);
    if (stale || keys.isEmpty() || (!containsKey(keyId) && canRefreshKeys())) {
      keys = loadKeys(configuration, provider);
      keysFingerprint = fingerprint;
      keysLoadedAt = System.currentTimeMillis();
    }
    if (keyId != null) {
      PublicKey key = keys.get(keyId);
      if (key == null) {
        throw new OidcException("The provider has no signing key with id " + keyId);
      }
      return key;
    }
    if (keys.size() != 1) {
      throw new OidcException("The ID token has no key id and the provider publishes " + keys.size() + " keys");
    }
    return keys.values().iterator().next();
  }

  private boolean containsKey(final String keyId) {
    return keyId == null || keys.containsKey(keyId);
  }

  private boolean canRefreshKeys() {
    return System.currentTimeMillis() - keysLoadedAt > JWKS_REFRESH_MIN_INTERVAL_MILLIS;
  }

  private Map<String, PublicKey> loadKeys(final OidcConfiguration configuration, final Metadata provider)
      throws OidcException
  {
    JsonNode jwks;
    if (!isNullOrEmpty(configuration.getIdpJwks())) {
      try {
        jwks = mapper.readTree(configuration.getIdpJwks());
      }
      catch (IOException e) {
        throw new OidcException("The configured JSON Web Keys are not valid JSON", e);
      }
    }
    else {
      jwks = getJson(configuration, provider.jwksUri(), null);
    }
    Map<String, PublicKey> result = new HashMap<>();
    JsonNode keyNodes = jwks.get("keys");
    if (keyNodes == null || !keyNodes.isArray()) {
      throw new OidcException("The JSON Web Key Set has no keys");
    }
    int index = 0;
    for (JsonNode jwk : keyNodes) {
      String use = text(jwk, "use");
      if (use != null && !"sig".equals(use)) {
        continue;
      }
      try {
        PublicKey key = parseKey(jwk);
        if (key != null) {
          String keyId = text(jwk, "kid");
          result.put(keyId != null ? keyId : "#" + index, key);
        }
      }
      catch (GeneralSecurityException | IllegalArgumentException e) {
        log.warn("Skipping unusable JSON Web Key {}: {}", text(jwk, "kid"), e.getMessage());
      }
      index++;
    }
    if (result.isEmpty()) {
      throw new OidcException("The JSON Web Key Set has no usable signing keys");
    }
    return Map.copyOf(result);
  }

  static PublicKey parseKey(final JsonNode jwk) throws GeneralSecurityException {
    String type = text(jwk, "kty");
    if ("RSA".equals(type)) {
      BigInteger modulus = new BigInteger(1, Base64.getUrlDecoder().decode(text(jwk, "n")));
      BigInteger exponent = new BigInteger(1, Base64.getUrlDecoder().decode(text(jwk, "e")));
      return KeyFactory.getInstance("RSA").generatePublic(new RSAPublicKeySpec(modulus, exponent));
    }
    if ("EC".equals(type)) {
      String curve = switch (Objects.requireNonNullElse(text(jwk, "crv"), "")) {
        case "P-256" -> "secp256r1";
        case "P-384" -> "secp384r1";
        case "P-521" -> "secp521r1";
        default -> throw new GeneralSecurityException("Unsupported curve " + text(jwk, "crv"));
      };
      AlgorithmParameters parameters = AlgorithmParameters.getInstance("EC");
      parameters.init(new ECGenParameterSpec(curve));
      ECParameterSpec spec = parameters.getParameterSpec(ECParameterSpec.class);
      ECPoint point = new ECPoint(
          new BigInteger(1, Base64.getUrlDecoder().decode(text(jwk, "x"))),
          new BigInteger(1, Base64.getUrlDecoder().decode(text(jwk, "y"))));
      return KeyFactory.getInstance("EC").generatePublic(new ECPublicKeySpec(point, spec));
    }
    return null;
  }

  static Algorithm algorithm(final String name, final PublicKey key) throws OidcException {
    try {
      return switch (name) {
        case "RS256" -> Algorithm.RSA256((RSAPublicKey) key, null);
        case "RS384" -> Algorithm.RSA384((RSAPublicKey) key, null);
        case "RS512" -> Algorithm.RSA512((RSAPublicKey) key, null);
        case "ES256" -> Algorithm.ECDSA256((ECPublicKey) key, null);
        case "ES384" -> Algorithm.ECDSA384((ECPublicKey) key, null);
        case "ES512" -> Algorithm.ECDSA512((ECPublicKey) key, null);
        default -> throw new OidcException("Unsupported ID token signature algorithm " + name);
      };
    }
    catch (ClassCastException e) {
      throw new OidcException("The signing key does not match the " + name + " algorithm", e);
    }
  }

  private JsonNode getJson(final OidcConfiguration configuration, final String url, final String authorization)
      throws OidcException
  {
    HttpGet get = new HttpGet(url);
    get.setHeader("Accept", "application/json");
    if (authorization != null) {
      get.setHeader("Authorization", authorization);
    }
    return execute(configuration, get);
  }

  private JsonNode execute(final OidcConfiguration configuration, final HttpUriRequest request) throws OidcException {
    try (CloseableHttpClient client = client(configuration);
         CloseableHttpResponse response = client.execute(request)) {
      int status = response.getStatusLine().getStatusCode();
      byte[] body = response.getEntity() == null ? new byte[0] : EntityUtils.toByteArray(response.getEntity());
      if (body.length > MAX_RESPONSE_BYTES) {
        throw new OidcException(request.getURI() + " returned an oversized response");
      }
      if (status < 200 || status >= 300) {
        throw new OidcException(request.getURI() + " returned HTTP " + status + errorDetails(body));
      }
      return mapper.readTree(body);
    }
    catch (IOException e) {
      throw new OidcException("Request to " + request.getURI() + " failed: " + e.getMessage(), e);
    }
  }

  private String errorDetails(final byte[] body) {
    try {
      JsonNode node = mapper.readTree(body);
      String error = text(node, "error");
      if (error != null) {
        String description = text(node, "error_description");
        return " (" + error + (description != null ? ": " + description : "") + ")";
      }
    }
    catch (IOException e) {
      log.debug("Error response is not JSON", e);
    }
    return "";
  }

  private CloseableHttpClient client(final OidcConfiguration configuration) {
    if (configuration.isUseTrustStore()) {
      return HttpClients.custom()
          .useSystemProperties()
          .setSSLContext(trustStore.get().getSSLContext())
          .build();
    }
    return httpClientManager.create();
  }

  private static String fingerprint(final OidcConfiguration configuration) {
    return String.join("\n",
        Objects.toString(configuration.getIssuerUrl(), ""),
        Objects.toString(configuration.getIdpAuthorizationUrl(), ""),
        Objects.toString(configuration.getIdpTokenUrl(), ""),
        Objects.toString(configuration.getIdpJwksUrl(), ""),
        Objects.toString(configuration.getIdpLogoutUrl(), ""),
        Objects.toString(configuration.getIdpJwks(), ""),
        Boolean.toString(configuration.isUseTrustStore()));
  }

  private static String formEncode(final String value) {
    return URLEncoder.encode(value, StandardCharsets.UTF_8);
  }

  static String stripTrailingSlash(final String value) {
    String trimmed = value.trim();
    while (trimmed.endsWith("/")) {
      trimmed = trimmed.substring(0, trimmed.length() - 1);
    }
    return trimmed;
  }

  private static String firstNonEmpty(final String first, final String second) {
    return !isNullOrEmpty(first) ? first.trim() : second;
  }

  private static String text(final JsonNode node, final String field) {
    if (node == null) {
      return null;
    }
    JsonNode value = node.get(field);
    return value == null || value.isNull() ? null : value.asText();
  }
}
