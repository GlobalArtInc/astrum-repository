package org.globalart.astrum.repository.swift.internal;

import java.util.Optional;

import javax.annotation.Nonnull;

import org.globalart.astrum.repository.IllegalOperationException;
import org.globalart.astrum.repository.InvalidContentException;
import org.globalart.astrum.repository.RedeployDisabledException;
import org.globalart.astrum.repository.http.HttpMethods;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Response;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.swift.internal.SwiftResponses.API_VERSION;
import static org.globalart.astrum.repository.swift.internal.SwiftResponses.CONTENT_VERSION;

@Component
public class SwiftApiHandler
    implements Handler
{
  private static final Logger log = LoggerFactory.getLogger(SwiftApiHandler.class);

  @Nonnull
  @Override
  public Response handle(@Nonnull final Context context) throws Exception {
    Optional<SwiftProblem> unsupported = ApiVersions.check(SwiftPaths.header(context, "Accept"));
    if (unsupported.isPresent()) {
      return SwiftResponses.problem(unsupported.get().getStatus(), unsupported.get().getMessage());
    }
    Response response = proceed(context);
    if (!response.getStatus().isSuccessful() && response.getPayload() == null
        && response.getStatus().getCode() != 401 && response.getStatus().getCode() >= 400) {
      Response problem = SwiftResponses.problem(response.getStatus().getCode(), response.getStatus().getMessage());
      response.getHeaders().forEach(header -> {
        if (!problem.getHeaders().contains(header.getKey())) {
          problem.getHeaders().set(header.getKey(), header.getValue());
        }
      });
      return problem;
    }
    if (!response.getHeaders().contains(CONTENT_VERSION)) {
      response.getHeaders().set(CONTENT_VERSION, API_VERSION);
    }
    return response;
  }

  private static Response proceed(final Context context) throws Exception {
    try {
      return context.proceed();
    }
    catch (SwiftProblem e) {
      log.debug("Swift request {} {} failed: {}", context.getRequest().getAction(), context.getRequest().getPath(),
          e.getMessage());
      return SwiftResponses.problem(e.getStatus(), e.getMessage());
    }
    catch (RedeployDisabledException e) {
      return SwiftResponses.problem(409, "a release with this version already exists and repository "
          + context.getRepository().getName() + " does not allow redeployment");
    }
    catch (IllegalOperationException e) {
      return SwiftResponses.problem(405, "publishing isn't supported: " + e.getMessage());
    }
    catch (InvalidContentException e) {
      boolean put = HttpMethods.PUT.equals(context.getRequest().getAction());
      return SwiftResponses.problem(put ? 422 : 404, e.getMessage());
    }
  }
}
