package org.globalart.astrum.repository.docker.internal.servlet;

import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.time.OffsetDateTime;
import java.time.ZoneOffset;
import java.time.format.DateTimeFormatter;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.TreeSet;

import javax.annotation.Nullable;

import org.globalart.astrum.common.collect.AttributesMap;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.docker.DockerConfig;
import org.globalart.astrum.repository.docker.DockerFormat;
import org.globalart.astrum.repository.docker.internal.DockerConnectorManager;
import org.globalart.astrum.repository.docker.internal.security.DockerAuth;
import org.globalart.astrum.repository.docker.internal.security.DockerTokenManager;
import org.globalart.astrum.repository.docker.internal.v2.DockerError;
import org.globalart.astrum.repository.docker.internal.v2.DockerPagination;
import org.globalart.astrum.repository.docker.internal.v2.DockerResponses;
import org.globalart.astrum.repository.http.HttpMethods;
import org.globalart.astrum.repository.httpbridge.internal.ViewServlet;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.view.Parameters;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.ViewFacet;
import org.globalart.astrum.security.anonymous.AnonymousConfiguration;
import org.globalart.astrum.security.anonymous.AnonymousManager;
import org.globalart.astrum.security.token.BearerTokenRealm;

import com.fasterxml.jackson.databind.JsonNode;
import jakarta.inject.Provider;
import jakarta.servlet.ServletException;
import jakarta.servlet.ServletRequest;
import jakarta.servlet.ServletResponse;
import jakarta.servlet.annotation.WebServlet;
import jakarta.servlet.http.HttpServlet;
import jakarta.servlet.http.HttpServletRequest;
import jakarta.servlet.http.HttpServletRequestWrapper;
import jakarta.servlet.http.HttpServletResponse;
import org.apache.shiro.SecurityUtils;
import org.apache.shiro.subject.PrincipalCollection;
import org.apache.shiro.subject.SimplePrincipalCollection;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

@WebServlet(urlPatterns = DockerV2Servlet.MOUNT_POINT)
@Component
public class DockerV2Servlet
    extends HttpServlet
{
  public static final String MOUNT_POINT = "/v2/*";

  private static final Logger log = LoggerFactory.getLogger(DockerV2Servlet.class);

  private static final String TOKEN_PATH = "/token";

  private static final String CATALOG_PATH = "/_catalog";

  private static final long TOKEN_LIFETIME_SECONDS = 3600;

  private final RepositoryManager repositoryManager;

  private final List<ViewServlet> viewServlets;

  private final DockerConnectorManager connectorManager;

  private final DockerAuth dockerAuth;

  private final DockerTokenManager tokenManager;

  private final Provider<AnonymousManager> anonymousManager;

  @Autowired
  public DockerV2Servlet(
      final RepositoryManager repositoryManager,
      final List<ViewServlet> viewServlets,
      final DockerConnectorManager connectorManager,
      final DockerAuth dockerAuth,
      final DockerTokenManager tokenManager,
      final Provider<AnonymousManager> anonymousManager)
  {
    this.repositoryManager = checkNotNull(repositoryManager);
    this.viewServlets = checkNotNull(viewServlets);
    this.connectorManager = checkNotNull(connectorManager);
    this.dockerAuth = checkNotNull(dockerAuth);
    this.tokenManager = checkNotNull(tokenManager);
    this.anonymousManager = checkNotNull(anonymousManager);
  }

  private ViewServlet viewServlet() {
    return viewServlets.stream()
        .filter(servlet -> servlet.getClass() == ViewServlet.class)
        .findFirst()
        .orElseGet(() -> viewServlets.get(0));
  }

  @Override
  protected void service(final HttpServletRequest request, final HttpServletResponse response)
      throws ServletException, IOException
  {
    String path = request.getPathInfo() == null ? "" : request.getPathInfo();
    String contextPath = request.getContextPath() == null ? "" : request.getContextPath();
    String realm = externalBase(request) + contextPath + "/v2/token";
    request.setAttribute(DockerAuth.REALM_ATTRIBUTE, realm);
    request.setAttribute(DockerAuth.SERVICE_ATTRIBUTE, host(request));

    if (TOKEN_PATH.equals(path)) {
      token(request, response);
      return;
    }

    String portRepository = connectorManager.repositoryForPort(request.getLocalPort());
    if (portRepository != null) {
      Repository repository = repositoryManager.get(portRepository);
      if (repository == null) {
        send(request, response, DockerResponses.error(DockerError.NAME_UNKNOWN));
        return;
      }
      if (path.isEmpty() || "/".equals(path)) {
        ping(request, response, repository);
        return;
      }
      forward(request, response, repository, path, contextPath + "/v2/", "");
      return;
    }

    if (path.isEmpty() || "/".equals(path)) {
      ping(request, response, null);
      return;
    }
    if (CATALOG_PATH.equals(path)) {
      catalog(request, response, contextPath);
      return;
    }

    String remainder = path.substring(1);
    int slash = remainder.indexOf('/');
    String repositoryName = slash < 0 ? remainder : remainder.substring(0, slash);
    String rest = slash < 0 ? "" : remainder.substring(slash);
    Repository repository = repositoryManager.get(repositoryName);
    if (repository == null || !DockerFormat.NAME.equals(repository.getFormat().getValue())
        || !DockerConfig.pathEnabled(repository)) {
      AttributesMap attributes = attributes(request);
      if (DockerAuth.isAnonymous() && !DockerAuth.isTokenAuthenticated(attributes)) {
        send(request, response, dockerAuth.unauthorized(null, attributes, null));
        return;
      }
      send(request, response, DockerResponses.error(DockerError.NAME_UNKNOWN,
          "repository " + repositoryName + " is not a docker repository reachable by path"));
      return;
    }
    if (rest.isEmpty() || "/".equals(rest)) {
      ping(request, response, repository);
      return;
    }
    forward(request, response, repository, rest, contextPath + "/v2/" + repositoryName + "/", repositoryName + "/");
  }

  private void forward(
      final HttpServletRequest request,
      final HttpServletResponse response,
      final Repository repository,
      final String rest,
      final String prefix,
      final String namePrefix) throws ServletException, IOException
  {
    request.setAttribute(DockerAuth.PREFIX_ATTRIBUTE, prefix);
    request.setAttribute(DockerAuth.NAME_PREFIX_ATTRIBUTE, namePrefix);
    String pathInfo = "/" + repository.getName() + "/v2" + rest;
    viewServlet().service((ServletRequest) new ForwardedRequest(request, pathInfo), (ServletResponse) response);
  }

  private void ping(
      final HttpServletRequest request,
      final HttpServletResponse response,
      @Nullable final Repository repository) throws IOException
  {
    AttributesMap attributes = attributes(request);
    boolean anonymous = DockerAuth.isAnonymous();
    boolean forceBasic = repository != null && DockerConfig.forceBasicAuth(repository);
    if ((anonymous && !DockerAuth.isTokenAuthenticated(attributes)) || (forceBasic && anonymous)) {
      send(request, response, dockerAuth.unauthorized(repository, attributes, null));
      return;
    }
    send(request, response, DockerResponses.jsonResponse(200, Map.of()));
  }

  private void token(final HttpServletRequest request, final HttpServletResponse response) throws IOException {
    if (!HttpMethods.GET.equals(request.getMethod()) && !HttpMethods.HEAD.equals(request.getMethod())) {
      send(request, response, DockerResponses.error(404, DockerError.UNSUPPORTED,
          "use GET with basic credentials to obtain a token", null));
      return;
    }
    if (!dockerAuth.tokenRealmEnabled()) {
      send(request, response, DockerResponses.error(404, DockerError.UNSUPPORTED,
          "the Docker Bearer Token Realm is not active", null));
      return;
    }
    PrincipalCollection principals;
    if (!DockerAuth.isAnonymous()) {
      principals = SecurityUtils.getSubject().getPrincipals();
    }
    else {
      AnonymousManager manager = anonymousManager.get();
      AnonymousConfiguration configuration = manager.getConfiguration();
      if (!manager.isEnabled() || configuration == null || configuration.getUserId() == null) {
        Response unauthorized = DockerResponses.error(DockerError.UNAUTHORIZED);
        unauthorized.getHeaders().replace(DockerResponses.WWW_AUTHENTICATE, DockerAuth.BASIC_CHALLENGE);
        send(request, response, unauthorized);
        return;
      }
      principals = new SimplePrincipalCollection(configuration.getUserId(), configuration.getRealmName());
    }
    String token = tokenManager.token(principals);
    Map<String, Object> body = new LinkedHashMap<>();
    body.put("token", token);
    body.put("access_token", token);
    body.put("expires_in", TOKEN_LIFETIME_SECONDS);
    body.put("issued_at", OffsetDateTime.now(ZoneOffset.UTC).format(DateTimeFormatter.ISO_OFFSET_DATE_TIME));
    send(request, response, DockerResponses.jsonResponse(200, body));
  }

  private void catalog(
      final HttpServletRequest request,
      final HttpServletResponse response,
      final String contextPath) throws IOException
  {
    AttributesMap attributes = attributes(request);
    if (DockerAuth.isAnonymous() && !DockerAuth.isTokenAuthenticated(attributes)) {
      send(request, response, dockerAuth.unauthorized(null, attributes, "registry:catalog:*"));
      return;
    }
    TreeSet<String> names = new TreeSet<>();
    for (Repository repository : repositoryManager.browse()) {
      if (!DockerFormat.NAME.equals(repository.getFormat().getValue())
          || !repository.getConfiguration().isOnline()
          || !DockerConfig.pathEnabled(repository)) {
        continue;
      }
      try {
        Request catalogRequest = new Request.Builder()
            .action(HttpMethods.GET)
            .path("/v2/_catalog")
            .attributes(new AttributesMap(new LinkedHashMap<>(attributes.backing())))
            .build();
        Response catalog = repository.facet(ViewFacet.class).dispatch(catalogRequest);
        if (!catalog.getStatus().isSuccessful() || catalog.getPayload() == null) {
          continue;
        }
        try (InputStream in = catalog.getPayload().openInputStream()) {
          JsonNode body = DockerResponses.MAPPER.readTree(in);
          for (JsonNode name : body.path("repositories")) {
            names.add(repository.getName() + "/" + name.asText());
          }
        }
      }
      catch (Exception e) {
        log.debug("Skipping docker repository {} in the catalog", repository.getName(), e);
      }
    }
    Parameters parameters = new Parameters();
    if (request.getParameter("n") != null) {
      parameters.set("n", request.getParameter("n"));
    }
    if (request.getParameter("last") != null) {
      parameters.set("last", request.getParameter("last"));
    }
    DockerPagination.Page page = DockerPagination.page(names, parameters, contextPath + "/v2/_catalog");
    Response result = DockerResponses.jsonResponse(200, Map.of("repositories", page.getItems()));
    if (page.getNextLink() != null) {
      result.getHeaders().replace("Link", page.getNextLink());
    }
    send(request, response, result);
  }

  private static AttributesMap attributes(final HttpServletRequest request) {
    AttributesMap attributes = new AttributesMap();
    for (String name : List.of(DockerAuth.REALM_ATTRIBUTE, DockerAuth.SERVICE_ATTRIBUTE,
        BearerTokenRealm.IS_TOKEN_AUTH_KEY)) {
      Object value = request.getAttribute(name);
      if (value != null) {
        attributes.set(name, value);
      }
    }
    return attributes;
  }

  private static void send(
      final HttpServletRequest request,
      final HttpServletResponse response,
      final Response view) throws IOException
  {
    response.setStatus(view.getStatus().getCode());
    view.getHeaders().forEach(header -> response.addHeader(header.getKey(), header.getValue()));
    if (view.getPayload() == null) {
      response.setContentLength(0);
      return;
    }
    try (var payload = view.getPayload()) {
      if (payload.getContentType() != null) {
        response.setContentType(payload.getContentType());
      }
      if (payload.getSize() >= 0) {
        response.setContentLengthLong(payload.getSize());
      }
      if (!HttpMethods.HEAD.equals(request.getMethod())) {
        try (InputStream in = payload.openInputStream(); OutputStream out = response.getOutputStream()) {
          in.transferTo(out);
        }
      }
    }
  }

  private static String externalBase(final HttpServletRequest request) {
    String scheme = request.getHeader("X-Forwarded-Proto");
    if (scheme == null || scheme.isBlank()) {
      scheme = request.getScheme();
    }
    else {
      scheme = scheme.split(",")[0].trim();
    }
    return scheme + "://" + host(request);
  }

  private static String host(final HttpServletRequest request) {
    String host = request.getHeader("X-Forwarded-Host");
    if (host == null || host.isBlank()) {
      host = request.getHeader("Host");
    }
    else {
      host = host.split(",")[0].trim();
    }
    if (host == null || host.isBlank()) {
      int port = request.getServerPort();
      host = request.getServerName() + (port > 0 ? ":" + port : "");
    }
    return host;
  }

  private static final class ForwardedRequest
      extends HttpServletRequestWrapper
  {
    private final String pathInfo;

    private ForwardedRequest(final HttpServletRequest request, final String pathInfo) {
      super(request);
      this.pathInfo = pathInfo;
    }

    @Override
    public String getPathInfo() {
      return pathInfo;
    }

    @Override
    public String getServletPath() {
      return "/repository";
    }

    @Override
    public String getRequestURI() {
      String contextPath = getContextPath() == null ? "" : getContextPath();
      return contextPath + "/repository" + pathInfo;
    }

    @Override
    public StringBuffer getRequestURL() {
      StringBuffer url = new StringBuffer();
      url.append(getScheme()).append("://").append(host((HttpServletRequest) getRequest()));
      url.append(getRequestURI());
      return url;
    }
  }
}
