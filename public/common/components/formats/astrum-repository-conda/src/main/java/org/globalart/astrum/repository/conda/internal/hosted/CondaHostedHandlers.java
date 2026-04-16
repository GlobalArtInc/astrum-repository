package org.globalart.astrum.repository.conda.internal.hosted;

import org.globalart.astrum.repository.conda.internal.metadata.InvalidCondaPackageException;
import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.PartPayload;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.repository.view.matchers.token.TokenMatcher.State;

import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.conda.util.CondaPathUtils.assetPath;
import static org.globalart.astrum.repository.conda.util.CondaPathUtils.matcherState;
import static org.globalart.astrum.repository.conda.util.CondaPathUtils.subdir;
import static org.globalart.astrum.repository.conda.util.CondaPathUtils.subdirPath;

@Component
public class CondaHostedHandlers
{
  public final Handler getPackage = context -> {
    Content content = context.getRepository().facet(CondaHostedFacet.class).getPackage(assetPath(context));
    return content != null ? HttpResponses.ok(content) : HttpResponses.notFound();
  };

  public final Handler getRepodata = context -> {
    State state = matcherState(context);
    Content content = context.getRepository().facet(CondaHostedFacet.class)
        .getRepodata(subdirPath(state), subdir(state));
    return content != null ? HttpResponses.ok(content) : HttpResponses.notFound();
  };

  public final Handler upload = context -> {
    Request request = context.getRequest();
    Payload payload = request.getPayload();
    if (request.isMultipart()) {
      payload = null;
      for (PartPayload part : request.getMultiparts()) {
        if (!part.isFormField()) {
          payload = part;
          break;
        }
      }
    }
    if (payload == null) {
      return HttpResponses.badRequest("Missing package content");
    }
    try {
      context.getRepository().facet(CondaHostedFacet.class).upload(assetPath(context), payload);
      return HttpResponses.created();
    }
    catch (InvalidCondaPackageException e) {
      return HttpResponses.badRequest(e.getMessage());
    }
  };
}
