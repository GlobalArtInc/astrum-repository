package org.globalart.astrum.repository.rubygems.internal;

import java.io.ByteArrayOutputStream;
import java.util.LinkedHashMap;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;

import javax.annotation.Nonnull;

import org.globalart.astrum.repository.http.HttpMethods;
import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.rubygems.internal.gem.GemCoordinates;
import org.globalart.astrum.repository.rubygems.internal.index.CompactIndex;
import org.globalart.astrum.repository.rubygems.internal.index.DependencyApi;
import org.globalart.astrum.repository.rubygems.internal.index.InfoEntry;
import org.globalart.astrum.repository.rubygems.internal.marshal.RubyMarshalWriter;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.ViewFacet;

import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.rubygems.internal.RubygemsResponses.BINARY;

@Component
public class RubygemsDependenciesHandler
    implements Handler
{
  @Nonnull
  @Override
  public Response handle(@Nonnull final Context context) throws Exception {
    Set<String> names = new LinkedHashSet<>();
    for (String parameter : context.getRequest().getParameters().getAll("gems")) {
      for (String name : parameter.split(",")) {
        String trimmed = name.strip();
        if (!trimmed.isEmpty()) {
          if (!GemCoordinates.isValidName(trimmed)) {
            throw RubygemsException.badRequest("Invalid gem name: " + trimmed);
          }
          names.add(trimmed);
        }
      }
    }
    if (names.size() > DependencyApi.MAX_GEMS) {
      throw new RubygemsException(422, "Too many gems (use less than " + DependencyApi.MAX_GEMS + " gems)");
    }
    if (names.isEmpty()) {
      ByteArrayOutputStream out = new ByteArrayOutputStream();
      new RubyMarshalWriter(out).array(0);
      return HttpResponses.ok(RubygemsResponses.content(out.toByteArray(), BINARY, null));
    }
    ViewFacet view = context.getRepository().facet(ViewFacet.class);
    Map<String, List<InfoEntry>> entries = new LinkedHashMap<>();
    for (String name : names) {
      Request request = new Request.Builder().action(HttpMethods.GET).path(RubygemsPaths.INFO_PREFIX + name).build();
      byte[] info = RubygemsResponses.read(view.dispatch(request, context));
      if (info != null) {
        entries.put(name, CompactIndex.parseInfo(info));
      }
    }
    return HttpResponses.ok(RubygemsResponses.content(DependencyApi.marshal(entries), BINARY, null));
  }
}
