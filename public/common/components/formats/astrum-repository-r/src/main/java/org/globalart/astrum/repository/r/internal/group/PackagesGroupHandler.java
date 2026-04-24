
package org.globalart.astrum.repository.r.internal.group;

import java.util.LinkedHashMap;
import java.util.Map;
import java.util.Map.Entry;

import javax.annotation.Nonnull;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.group.GroupHandler;
import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.http.HttpStatus;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.Response;

import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.http.HttpConditions.isConditional;
import static org.globalart.astrum.repository.http.HttpConditions.makeConditional;
import static org.globalart.astrum.repository.http.HttpConditions.makeUnconditional;
import static org.globalart.astrum.repository.r.internal.util.RPathUtils.requestPath;

@Component
public class PackagesGroupHandler
    extends GroupHandler
{
  @Override
  protected Response doGet(@Nonnull final Context context, @Nonnull final DispatchedRepositories dispatched)
      throws Exception
  {
    RGroupFacet groupFacet = context.getRepository().facet(RGroupFacet.class);
    Map<Repository, Response> responses;
    boolean conditional = isConditional(context.getRequest());
    if (conditional) {
      makeUnconditional(context.getRequest());
    }
    try {
      responses = getAll(context, groupFacet.members(), dispatched);
    }
    finally {
      if (conditional) {
        makeConditional(context.getRequest());
      }
    }
    Map<Repository, Content> packages = new LinkedHashMap<>();
    for (Entry<Repository, Response> entry : responses.entrySet()) {
      Response response = entry.getValue();
      Payload payload = response.getPayload();
      if (response.getStatus().getCode() == HttpStatus.OK && payload != null) {
        packages.put(entry.getKey(), payload instanceof Content content ? content : new Content(payload));
      }
    }
    if (packages.isEmpty()) {
      return notFoundResponse(context);
    }
    return HttpResponses.ok(groupFacet.mergedPackages(requestPath(context), packages));
  }
}
