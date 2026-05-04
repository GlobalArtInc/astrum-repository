package org.globalart.astrum.repository.cargo.internal.group;

import java.util.ArrayList;
import java.util.List;

import javax.annotation.Nonnull;

import org.globalart.astrum.repository.cargo.internal.CargoResponses;
import org.globalart.astrum.repository.cargo.internal.CargoSearch;
import org.globalart.astrum.repository.group.GroupFacet;
import org.globalart.astrum.repository.group.GroupHandler;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Response;

import com.fasterxml.jackson.databind.JsonNode;
import org.springframework.stereotype.Component;

@Component
public class CargoGroupSearchHandler
    extends GroupHandler
{
  @Nonnull
  @Override
  protected Response doGet(
      @Nonnull final Context context,
      @Nonnull final DispatchedRepositories dispatched) throws Exception
  {
    GroupFacet groupFacet = context.getRepository().facet(GroupFacet.class);
    List<JsonNode> results = new ArrayList<>();
    for (Response response : getAll(context, groupFacet.members(), dispatched).values()) {
      byte[] body = CargoGroupIndexHandler.read(response);
      if (body != null) {
        try {
          results.add(CargoResponses.read(body));
        }
        catch (RuntimeException e) {
          log.debug("Ignoring unreadable search result in {}", context.getRepository().getName(), e);
        }
      }
    }
    return CargoResponses.json(CargoSearch.merge(results, CargoSearch.perPage(context)));
  }
}
