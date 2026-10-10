package org.globalart.astrum.oidc.internal;

import java.io.IOException;
import java.net.URLEncoder;
import java.nio.charset.StandardCharsets;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.security.SecureRandom;
import java.util.Base64;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.concurrent.TimeUnit;

import jakarta.servlet.annotation.WebServlet;
import jakarta.servlet.http.Cookie;
import jakarta.servlet.http.HttpServlet;
import jakarta.servlet.http.HttpServletRequest;
import jakarta.servlet.http.HttpServletResponse;

import com.google.common.cache.Cache;
import com.google.common.cache.CacheBuilder;
import org.globalart.astrum.common.app.BaseUrlHolder;
import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.security.JwtHelper;
import org.globalart.astrum.security.anonymous.AnonymousManager;
import org.globalart.astrum.security.authc.LoginEvent;
import org.globalart.astrum.security.authz.AuthorizationConfigurationChanged;
import org.globalart.astrum.security.realm.RealmManager;
import org.apache.shiro.authc.AuthenticationException;
import org.apache.shiro.web.mgt.WebSecurityManager;
import org.apache.shiro.web.subject.WebSubject;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.ObjectProvider;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.base.Strings.isNullOrEmpty;

@WebServlet(urlPatterns = {OidcServlet.LOGIN_PATH, OidcServlet.CALLBACK_PATH})
@Component
public class OidcServlet
    extends HttpServlet
{
  public static final String LOGIN_PATH = "/oidc/login";

  public static final String CALLBACK_PATH = "/oidc/callback";

  static final String STATE_COOKIE = "ASTRUMOIDCSTATE";

  private static final Logger log = LoggerFactory.getLogger(OidcServlet.class);

  private static final int STATE_TTL_SECONDS = 600;

  private final OidcConfigurationStore configurationStore;

  private final OidcProvider provider;

  private final OidcUserStore users;

  private final RealmManager realmManager;

  private final WebSecurityManager securityManager;

  private final EventManager eventManager;

  private final ObjectProvider<JwtHelper> jwtHelper;

  private final ObjectProvider<AnonymousManager> anonymousManager;

  private final SecureRandom random = new SecureRandom();

  private final Cache<String, PendingLogin> pending = CacheBuilder.newBuilder()
      .expireAfterWrite(STATE_TTL_SECONDS, TimeUnit.SECONDS)
      .maximumSize(10_000)
      .build();

  record PendingLogin(String nonce, String codeVerifier, String redirectUri, String returnHash)
  {
  }

  @Autowired
  public OidcServlet(
      final OidcConfigurationStore configurationStore,
      final OidcProvider provider,
      final OidcUserStore users,
      final RealmManager realmManager,
      final WebSecurityManager securityManager,
      final EventManager eventManager,
      final ObjectProvider<JwtHelper> jwtHelper,
      final ObjectProvider<AnonymousManager> anonymousManager)
  {
    this.configurationStore = checkNotNull(configurationStore);
    this.provider = checkNotNull(provider);
    this.users = checkNotNull(users);
    this.realmManager = checkNotNull(realmManager);
    this.securityManager = checkNotNull(securityManager);
    this.eventManager = checkNotNull(eventManager);
    this.jwtHelper = checkNotNull(jwtHelper);
    this.anonymousManager = checkNotNull(anonymousManager);
  }

  @Override
  protected void doGet(final HttpServletRequest request, final HttpServletResponse response) throws IOException {
    response.setHeader("Cache-Control", "no-store");
    Optional<OidcConfiguration> configuration = configurationStore.load();
    if (configuration.isEmpty() || !realmManager.isRealmEnabled(OidcRealm.NAME)) {
      response.sendError(HttpServletResponse.SC_NOT_FOUND, "OpenID Connect sign-in is not enabled");
      return;
    }
    try {
      if (request.getServletPath().equals(CALLBACK_PATH)) {
        callback(configuration.get(), request, response);
      }
      else {
        login(configuration.get(), request, response);
      }
    }
    catch (OidcException e) {
      log.warn("OpenID Connect sign-in failed: {}", e.getMessage());
      log.debug("OpenID Connect sign-in failure", e);
      response.sendError(HttpServletResponse.SC_FORBIDDEN, "Sign-in failed: " + e.getMessage());
    }
  }

  private void login(
      final OidcConfiguration configuration,
      final HttpServletRequest request,
      final HttpServletResponse response) throws OidcException, IOException
  {
    OidcProvider.Metadata metadata = provider.metadata(configuration);
    String state = randomToken();
    String nonce = randomToken();
    String codeVerifier = randomToken() + randomToken();
    String redirectUri = baseUrl(request) + CALLBACK_PATH;
    pending.put(state, new PendingLogin(nonce, codeVerifier, redirectUri, returnHash(request.getParameter("hash"))));

    Map<String, String> parameters = new LinkedHashMap<>();
    parameters.put("response_type", "code");
    parameters.put("client_id", configuration.getClientId());
    parameters.put("redirect_uri", redirectUri);
    parameters.put("scope", "openid profile email");
    parameters.put("state", state);
    parameters.put("nonce", nonce);
    parameters.put("code_challenge", codeChallenge(codeVerifier));
    parameters.put("code_challenge_method", "S256");
    parameters.putAll(configuration.getAuthorizationCustomParams());

    Cookie cookie = new Cookie(STATE_COOKIE, state);
    cookie.setHttpOnly(true);
    cookie.setSecure(request.isSecure());
    cookie.setPath(cookiePath(request));
    cookie.setMaxAge(STATE_TTL_SECONDS);
    response.addCookie(cookie);
    response.sendRedirect(appendQuery(metadata.authorizationEndpoint(), parameters));
  }

  private void callback(
      final OidcConfiguration configuration,
      final HttpServletRequest request,
      final HttpServletResponse response) throws OidcException, IOException
  {
    String state = request.getParameter("state");
    String cookieState = cookieValue(request, STATE_COOKIE);
    expireStateCookie(request, response);
    if (isNullOrEmpty(state) || !state.equals(cookieState)) {
      throw new OidcException("The sign-in request expired or was started in another browser, please try again");
    }
    PendingLogin login = pending.getIfPresent(state);
    pending.invalidate(state);
    if (login == null) {
      throw new OidcException("The sign-in request expired, please try again");
    }
    String error = request.getParameter("error");
    if (!isNullOrEmpty(error)) {
      String description = request.getParameter("error_description");
      throw new OidcException("The identity provider returned " + error + (description != null ? ": " + description : ""));
    }
    String code = request.getParameter("code");
    if (isNullOrEmpty(code)) {
      throw new OidcException("The identity provider did not return an authorization code");
    }

    char[] clientSecret = configurationStore.clientSecret(configuration);
    OidcProvider.Tokens tokens = provider.exchangeCode(
        configuration, clientSecret, code, login.redirectUri(), login.codeVerifier());
    Map<String, Object> claims = new LinkedHashMap<>(provider.verifyIdToken(configuration, tokens.idToken(), login.nonce()));
    mergeUserInfo(configuration, tokens, claims);

    OidcClaims.checkExactMatch(configuration, claims);
    String userId = OidcClaims.userId(configuration, claims);
    rejectReservedUserId(userId);
    List<String> groups = isNullOrEmpty(configuration.getGroupsClaim())
        ? List.of()
        : OidcClaims.strings(claims, configuration.getGroupsClaim());
    OidcUser user = new OidcUser(
        userId,
        OidcClaims.string(claims, "sub"),
        firstName(configuration, claims),
        lastName(configuration, claims),
        OidcClaims.claimOrDefault(claims, configuration.getEmailClaim(), OidcConfiguration.DEFAULT_EMAIL_CLAIM),
        groups,
        System.currentTimeMillis());
    if (users.put(user)) {
      eventManager.post(new AuthorizationConfigurationChanged());
    }

    WebSubject subject = new WebSubject.Builder(securityManager, request, response).buildWebSubject();
    try {
      subject.login(new OidcAuthenticationToken(userId, user.subject()));
    }
    catch (AuthenticationException e) {
      throw new OidcException("Astrum rejected the sign-in: " + e.getMessage(), e);
    }
    JwtHelper helper = jwtHelper.getIfAvailable();
    if (helper != null) {
      response.addCookie(helper.createJwtCookie(subject, request.isSecure()));
    }
    eventManager.post(new LoginEvent(userId, OidcRealm.NAME));
    log.info("User {} signed in with OpenID Connect", userId);

    String target = request.getContextPath() + "/";
    if (!isNullOrEmpty(login.returnHash())) {
      target += "#" + login.returnHash();
    }
    response.sendRedirect(target);
  }

  private void mergeUserInfo(
      final OidcConfiguration configuration,
      final OidcProvider.Tokens tokens,
      final Map<String, Object> claims)
  {
    try {
      Map<String, Object> userInfo = provider.userInfo(configuration, tokens.accessToken());
      Object subject = userInfo.get("sub");
      if (subject != null && subject.equals(claims.get("sub"))) {
        userInfo.forEach(claims::putIfAbsent);
      }
    }
    catch (OidcException e) {
      log.warn("Could not load the OpenID Connect user info, using the ID token claims only: {}", e.getMessage());
    }
  }

  private void rejectReservedUserId(final String userId) throws OidcException {
    AnonymousManager manager = anonymousManager.getIfAvailable();
    String anonymousUserId = manager != null ? manager.getConfiguration().getUserId() : "anonymous";
    if (userId.equals(anonymousUserId)) {
      throw new OidcException("The username '" + userId + "' is reserved");
    }
  }

  private static String firstName(final OidcConfiguration configuration, final Map<String, Object> claims) {
    String value = OidcClaims.claimOrDefault(
        claims, configuration.getFirstNameClaim(), OidcConfiguration.DEFAULT_FIRST_NAME_CLAIM);
    return value != null ? value : OidcClaims.string(claims, "name");
  }

  private static String lastName(final OidcConfiguration configuration, final Map<String, Object> claims) {
    return OidcClaims.claimOrDefault(claims, configuration.getLastNameClaim(), OidcConfiguration.DEFAULT_LAST_NAME_CLAIM);
  }

  static String baseUrl(final HttpServletRequest request) {
    if (BaseUrlHolder.isSet()) {
      return OidcProvider.stripTrailingSlash(BaseUrlHolder.get());
    }
    StringBuilder url = new StringBuilder(request.getScheme()).append("://").append(request.getServerName());
    int port = request.getServerPort();
    boolean defaultPort = ("http".equals(request.getScheme()) && port == 80)
        || ("https".equals(request.getScheme()) && port == 443);
    if (!defaultPort) {
      url.append(':').append(port);
    }
    return url.append(request.getContextPath()).toString();
  }

  static String returnHash(final String encoded) {
    if (isNullOrEmpty(encoded)) {
      return null;
    }
    String hash = encoded.trim();
    try {
      String decoded = new String(Base64.getDecoder().decode(hash), StandardCharsets.UTF_8);
      if (decoded.startsWith("#") || decoded.startsWith("/")) {
        hash = decoded;
      }
    }
    catch (IllegalArgumentException e) {
      log.debug("returnTo is not Base64, using it as is");
    }
    if (hash.startsWith("#") || hash.startsWith("/")) {
      hash = hash.substring(1);
    }
    if (hash.isBlank() || hash.length() > 2048 || hash.chars().anyMatch(c -> c < 0x20 || c == 0x7f)) {
      return null;
    }
    return hash;
  }

  private String randomToken() {
    byte[] bytes = new byte[32];
    random.nextBytes(bytes);
    return Base64.getUrlEncoder().withoutPadding().encodeToString(bytes);
  }

  static String codeChallenge(final String verifier) {
    try {
      byte[] digest = MessageDigest.getInstance("SHA-256").digest(verifier.getBytes(StandardCharsets.US_ASCII));
      return Base64.getUrlEncoder().withoutPadding().encodeToString(digest);
    }
    catch (NoSuchAlgorithmException e) {
      throw new IllegalStateException(e);
    }
  }

  static String appendQuery(final String url, final Map<String, String> parameters) {
    StringBuilder result = new StringBuilder(url);
    char separator = url.contains("?") ? '&' : '?';
    for (Map.Entry<String, String> entry : parameters.entrySet()) {
      result.append(separator)
          .append(URLEncoder.encode(entry.getKey(), StandardCharsets.UTF_8))
          .append('=')
          .append(URLEncoder.encode(entry.getValue(), StandardCharsets.UTF_8));
      separator = '&';
    }
    return result.toString();
  }

  private static String cookiePath(final HttpServletRequest request) {
    return request.getContextPath() + "/oidc";
  }

  private static String cookieValue(final HttpServletRequest request, final String name) {
    Cookie[] cookies = request.getCookies();
    if (cookies == null) {
      return null;
    }
    for (Cookie cookie : cookies) {
      if (name.equals(cookie.getName())) {
        return cookie.getValue();
      }
    }
    return null;
  }

  private static void expireStateCookie(final HttpServletRequest request, final HttpServletResponse response) {
    Cookie cookie = new Cookie(STATE_COOKIE, "");
    cookie.setHttpOnly(true);
    cookie.setSecure(request.isSecure());
    cookie.setPath(cookiePath(request));
    cookie.setMaxAge(0);
    response.addCookie(cookie);
  }
}
