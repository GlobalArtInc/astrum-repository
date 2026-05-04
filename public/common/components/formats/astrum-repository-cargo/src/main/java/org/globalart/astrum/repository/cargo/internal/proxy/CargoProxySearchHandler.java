package org.globalart.astrum.repository.cargo.internal.proxy;

import java.io.IOException;
import java.util.Optional;

import javax.annotation.Nonnull;

import org.globalart.astrum.repository.cargo.internal.CargoResponses;
import org.globalart.astrum.repository.cargo.internal.CargoSearch;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Response;

import com.fasterxml.jackson.databind.JsonNode;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.stereotype.Component;

@Component
public class CargoProxySearchHandler
    implements Handler
{
  private static final Logger log = LoggerFactory.getLogger(CargoProxySearchHandler.class);

  @Nonnull
  @Override
  public Response handle(@Nonnull final Context context) {
    CargoProxyFacet proxy = context.getRepository().facet(CargoProxyFacet.class);
    try {
      Optional<JsonNode> result = proxy.remoteSearch(CargoSearch.query(context), CargoSearch.perPage(context));
      if (result.isPresent()) {
        return CargoResponses.json(result.get());
      }
    }
    catch (IOException | RuntimeException e) {
      log.warn("Remote search failed for repository {}: {}", context.getRepository().getName(), e.toString());
    }
    return CargoResponses.error(502, "search is unavailable on the remote of " + context.getRepository().getName());
  }
}
