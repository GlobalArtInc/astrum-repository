package org.globalart.astrum.repository.cargo.internal;

import javax.annotation.Nonnull;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Response;

import com.fasterxml.jackson.databind.node.ObjectNode;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.view.ContentTypes.APPLICATION_JSON;

@Component
public class CargoConfigHandler
    implements Handler
{
  @Nonnull
  @Override
  public Response handle(@Nonnull final Context context) {
    Repository repository = context.getRepository();
    String url = repository.getUrl();
    ObjectNode config = CargoResponses.object();
    config.put("dl", url + CargoPaths.CRATES_API);
    config.put("api", url);
    config.put("auth-required",
        CargoRepositoryConfig.requireAuthentication(repository) || !CargoRepositoryConfig.isAnonymous());
    return HttpResponses.ok(CargoResponses.content(CargoResponses.bytes(config), APPLICATION_JSON));
  }
}
