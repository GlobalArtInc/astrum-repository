package org.globalart.astrum.repository.alpine.internal.group;

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

import static org.globalart.astrum.repository.alpine.internal.AlpinePathUtils.requestPath;
import static org.globalart.astrum.repository.http.HttpConditions.isConditional;
import static org.globalart.astrum.repository.http.HttpConditions.makeConditional;
import static org.globalart.astrum.repository.http.HttpConditions.makeUnconditional;

@Component
public class AlpineGroupIndexHandler
    extends GroupHandler
{
  @Override
  protected Response doGet(@Nonnull final Context context, @Nonnull final DispatchedRepositories dispatched)
      throws Exception
  {
    AlpineGroupFacet groupFacet = context.getRepository().facet(AlpineGroupFacet.class);
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
    Map<Repository, Content> indexes = new LinkedHashMap<>();
    for (Entry<Repository, Response> entry : responses.entrySet()) {
      Response response = entry.getValue();
      Payload payload = response.getPayload();
      if (response.getStatus().getCode() == HttpStatus.OK && payload != null) {
        indexes.put(entry.getKey(), payload instanceof Content content ? content : new Content(payload));
      }
    }
    if (indexes.isEmpty()) {
      return notFoundResponse(context);
    }
    return HttpResponses.ok(groupFacet.mergedIndex(requestPath(context), indexes));
  }
}
