package org.globalart.astrum.repository.docker.internal.v2;

import javax.annotation.Nonnull;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.docker.DockerConfig;
import org.globalart.astrum.repository.docker.internal.security.DockerAuth;
import org.globalart.astrum.repository.http.HttpMethods;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.Status;

import org.apache.shiro.authz.AuthorizationException;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

@Component
public class DockerRequestHandler
    implements Handler
{
  public static final String AUTHORIZED_KEY = "security.authorized";

  private static final Logger log = LoggerFactory.getLogger(DockerRequestHandler.class);

  private final DockerAuth dockerAuth;

  @Autowired
  public DockerRequestHandler(final DockerAuth dockerAuth) {
    this.dockerAuth = checkNotNull(dockerAuth);
  }

  @Nonnull
  @Override
  public Response handle(@Nonnull final Context context) throws Exception {
    Repository repository = context.getRepository();
    boolean nested = context.getAttributes().contains(AUTHORIZED_KEY);
    if (!nested && DockerConfig.forceBasicAuth(repository) && DockerAuth.isAnonymous()) {
      return decorate(context, unauthorized(context));
    }
    Response response;
    try {
      response = context.proceed();
    }
    catch (AuthorizationException e) {
      response = DockerAuth.isAnonymous()
          ? unauthorized(context)
          : DockerResponses.error(DockerError.DENIED);
    }
    return decorate(context, response);
  }

  private Response unauthorized(final Context context) {
    DockerRoute route = context.getAttributes().get(DockerRoute.class);
    String scope = null;
    if (route != null && route.getName() != null) {
      String action = context.getRequest().getAction();
      boolean push = !HttpMethods.GET.equals(action) && !HttpMethods.HEAD.equals(action);
      scope = DockerLocations.scope(context, route.getName(), push);
    }
    return dockerAuth.unauthorized(context.getRepository(), context.getRequest().getAttributes(), scope);
  }

  private Response decorate(final Context context, final Response response) {
    Status status = response.getStatus();
    if (status.isSuccessful() || status.getCode() < 400
        || response.getAttributes().contains(DockerResponses.ERROR_ATTRIBUTE)) {
      response.getHeaders().replace(DockerResponses.API_VERSION_HEADER, DockerResponses.API_VERSION);
      return response;
    }
    int code = status.getCode();
    String message = status.getMessage();
    DockerError error = errorFor(context, code);
    Response converted = DockerResponses.error(code, error, message == null || message.isBlank()
        ? error.message()
        : message, null);
    if (code == 401) {
      String challenge = response.getHeaders().get(DockerResponses.WWW_AUTHENTICATE);
      if (challenge == null) {
        return decorate(context, unauthorized(context));
      }
      converted.getHeaders().replace(DockerResponses.WWW_AUTHENTICATE, challenge);
    }
    String retryAfter = response.getHeaders().get("Retry-After");
    if (retryAfter != null) {
      converted.getHeaders().replace("Retry-After", retryAfter);
    }
    closeQuietly(response);
    return converted;
  }

  private static void closeQuietly(final Response response) {
    if (response.getPayload() == null) {
      return;
    }
    try {
      response.getPayload().close();
    }
    catch (Exception e) {
      log.debug("Unable to close payload of a replaced response", e);
    }
  }

  private static DockerError errorFor(final Context context, final int code) {
    DockerRoute route = context.getAttributes().get(DockerRoute.class);
    DockerRoute.Type type = route == null ? null : route.getType();
    switch (code) {
      case 401:
        return DockerError.UNAUTHORIZED;
      case 403:
      case 409:
        return DockerError.DENIED;
      case 405:
        return DockerError.UNSUPPORTED;
      case 404:
        if (type == DockerRoute.Type.MANIFEST) {
          return DockerError.MANIFEST_UNKNOWN;
        }
        if (type == DockerRoute.Type.BLOB) {
          return DockerError.BLOB_UNKNOWN;
        }
        if (type == DockerRoute.Type.UPLOAD) {
          return DockerError.BLOB_UPLOAD_UNKNOWN;
        }
        return DockerError.NAME_UNKNOWN;
      case 400:
        if (type == DockerRoute.Type.MANIFEST) {
          return DockerError.MANIFEST_INVALID;
        }
        if (type == DockerRoute.Type.UPLOAD || type == DockerRoute.Type.UPLOADS) {
          return DockerError.BLOB_UPLOAD_INVALID;
        }
        return DockerError.UNSUPPORTED;
      case 429:
        return DockerError.TOOMANYREQUESTS;
      default:
        return DockerError.UNKNOWN;
    }
  }
}
