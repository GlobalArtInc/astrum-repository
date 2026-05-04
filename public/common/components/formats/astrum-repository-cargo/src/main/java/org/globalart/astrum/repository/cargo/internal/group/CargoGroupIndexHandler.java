package org.globalart.astrum.repository.cargo.internal.group;

import java.io.IOException;
import java.io.InputStream;
import java.util.ArrayList;
import java.util.List;

import javax.annotation.Nonnull;
import javax.annotation.Nullable;

import org.globalart.astrum.repository.cargo.internal.CargoIndex;
import org.globalart.astrum.repository.cargo.internal.CargoResponses;
import org.globalart.astrum.repository.group.GroupFacet;
import org.globalart.astrum.repository.group.GroupHandler;
import org.globalart.astrum.repository.http.HttpMethods;
import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.repository.view.Response;

import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.view.ContentTypes.TEXT_PLAIN;

@Component
public class CargoGroupIndexHandler
    extends GroupHandler
{
  @Nonnull
  @Override
  protected Response doGet(
      @Nonnull final Context context,
      @Nonnull final DispatchedRepositories dispatched) throws Exception
  {
    GroupFacet groupFacet = context.getRepository().facet(GroupFacet.class);
    Request request = new Request.Builder().copy(context.getRequest()).action(HttpMethods.GET).build();
    List<byte[]> files = new ArrayList<>();
    for (Response response : getAll(request, context, groupFacet.members(), dispatched).values()) {
      byte[] file = read(response);
      if (file != null) {
        files.add(file);
      }
    }
    if (files.isEmpty()) {
      return notFoundResponse(context);
    }
    return HttpResponses.ok(CargoResponses.content(CargoIndex.merge(files), TEXT_PLAIN));
  }

  @Nullable
  static byte[] read(final Response response) throws IOException {
    Payload payload = response.getPayload();
    if (!response.getStatus().isSuccessful() || payload == null) {
      return null;
    }
    try (Payload closeable = payload; InputStream in = closeable.openInputStream()) {
      return in.readAllBytes();
    }
  }
}
