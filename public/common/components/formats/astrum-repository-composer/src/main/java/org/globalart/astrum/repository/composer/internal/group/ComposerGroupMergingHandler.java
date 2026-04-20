
package org.globalart.astrum.repository.composer.internal.group;

import java.util.List;
import java.util.Map;
import java.util.stream.Collectors;

import javax.annotation.Nonnull;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.group.GroupFacet;
import org.globalart.astrum.repository.group.GroupHandler;
import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.http.HttpStatus;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.Response;

import static org.globalart.astrum.repository.http.HttpConditions.makeConditional;
import static org.globalart.astrum.repository.http.HttpConditions.makeUnconditional;

/**
 * Abstract handler for merging in the context of a Composer group repository, with merging left to concrete
 * implementations of the class.
 */
public abstract class ComposerGroupMergingHandler
    extends GroupHandler
{
  @Override
  protected final Response doGet(@Nonnull final Context context,
                                 @Nonnull final GroupHandler.DispatchedRepositories dispatched)
      throws Exception
  {
    Repository repository = context.getRepository();
    GroupFacet groupFacet = repository.facet(GroupFacet.class);

    makeUnconditional(context.getRequest());
    Map<Repository, Response> responses;
    try {
      responses = getAll(context, groupFacet.members(), dispatched);
    }
    finally {
      makeConditional(context.getRequest());
    }

    List<Payload> payloads = responses.values().stream()
        .filter(response -> response.getStatus().getCode() == HttpStatus.OK && response.getPayload() != null)
        .map(Response::getPayload)
        .collect(Collectors.toList());
    if (payloads.isEmpty()) {
      return notFoundResponse(context);
    }
    return HttpResponses.ok(merge(repository, payloads));
  }

  protected abstract Content merge(final Repository repository, final List<Payload> payloads) throws Exception;
}
