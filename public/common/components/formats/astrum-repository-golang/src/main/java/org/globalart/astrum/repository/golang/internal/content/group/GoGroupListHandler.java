package org.globalart.astrum.repository.golang.internal.content.group;

import java.io.IOException;
import java.io.InputStream;
import java.util.LinkedHashSet;
import java.util.Set;

import javax.annotation.Nonnull;

import org.globalart.astrum.repository.group.GroupFacet;
import org.globalart.astrum.repository.group.GroupHandler;
import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.payloads.BytesPayload;

import org.springframework.stereotype.Component;

import static java.nio.charset.StandardCharsets.UTF_8;
import static org.globalart.astrum.repository.view.ContentTypes.TEXT_PLAIN;

@Component
public class GoGroupListHandler
    extends GroupHandler
{
  @Nonnull
  @Override
  protected Response doGet(
      @Nonnull final Context context,
      @Nonnull final DispatchedRepositories dispatched) throws Exception
  {
    GroupFacet groupFacet = context.getRepository().facet(GroupFacet.class);
    Set<String> versions = new LinkedHashSet<>();
    boolean found = false;
    for (Response response : getAll(context, groupFacet.members(), dispatched).values()) {
      Payload payload = response.getPayload();
      if (!response.getStatus().isSuccessful() || payload == null) {
        continue;
      }
      found = true;
      versions.addAll(read(payload));
    }
    if (!found) {
      return notFoundResponse(context);
    }
    StringBuilder list = new StringBuilder();
    versions.forEach(version -> list.append(version).append('\n'));
    return HttpResponses.ok(new Content(new BytesPayload(list.toString().getBytes(UTF_8), TEXT_PLAIN)));
  }

  private Set<String> read(final Payload payload) throws IOException {
    Set<String> versions = new LinkedHashSet<>();
    try (Payload closeable = payload; InputStream in = closeable.openInputStream()) {
      for (String line : new String(in.readAllBytes(), UTF_8).split("\n")) {
        String version = line.trim();
        if (!version.isEmpty()) {
          versions.add(version);
        }
      }
    }
    return versions;
  }
}
