package org.globalart.astrum.repository.rubygems.internal.hosted;

import java.io.IOException;
import java.io.InputStream;
import java.net.URLDecoder;
import java.util.LinkedHashMap;
import java.util.Map;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.http.HttpMethods;
import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.rubygems.internal.RubygemsContentFacet;
import org.globalart.astrum.repository.rubygems.internal.RubygemsException;
import org.globalart.astrum.repository.rubygems.internal.RubygemsPaths;
import org.globalart.astrum.repository.rubygems.internal.RubygemsResponses;
import org.globalart.astrum.repository.rubygems.internal.gem.GemCoordinates;
import org.globalart.astrum.repository.rubygems.internal.gem.GemPlatform;
import org.globalart.astrum.repository.rubygems.internal.gem.GemSpec;
import org.globalart.astrum.repository.rubygems.internal.gem.GemVersion;
import org.globalart.astrum.repository.rubygems.internal.index.LegacySpecs;
import org.globalart.astrum.repository.security.SecurityFacet;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.Request;

import org.springframework.stereotype.Component;

import static java.nio.charset.StandardCharsets.UTF_8;

@Component
public class RubygemsHostedHandlers
{
  private static final int MAX_FORM_SIZE = 64 * 1024;

  public final Handler versions = context -> HttpResponses.ok(hosted(context).versions());

  public final Handler names = context -> HttpResponses.ok(hosted(context).names());

  public final Handler legacySpecs = context -> HttpResponses.ok(
      hosted(context).specs(LegacySpecs.Kind.fromPath(context.getRequest().getPath())));

  public final Handler info = context -> hosted(context).info(RubygemsPaths.gemName(context))
      .map(HttpResponses::ok)
      .orElseGet(() -> HttpResponses.notFound("gem " + RubygemsPaths.gemName(context) + " not found"));

  public final Handler quickSpec = context -> hosted(context).quickSpec(RubygemsPaths.fullName(context))
      .map(HttpResponses::ok)
      .orElseGet(HttpResponses::notFound);

  public final Handler gem = context -> hosted(context).gem(context.getRequest().getPath())
      .map(HttpResponses::ok)
      .orElseGet(HttpResponses::notFound);

  public final Handler push = context -> {
    Payload payload = context.getRequest().getPayload();
    if (payload == null || payload.getSize() == 0) {
      throw RubygemsException.badRequest("Missing gem file in the request body");
    }
    FluentAsset asset = hosted(context).push(payload, path -> authorize(context, HttpMethods.POST, path));
    GemCoordinates coordinates = GemSpec.fromAttributes(asset.attributes(RubygemsContentFacet.ATTRIBUTES).backing())
        .coordinates();
    return RubygemsResponses.text(200, "Successfully registered gem: " + coordinates.name() + " ("
        + coordinates.versionAndPlatform() + ")");
  };

  public final Handler yank = context -> {
    Map<String, String> form = form(context.getRequest());
    String name = form.get("gem_name");
    String version = form.get("version");
    String platform = form.get("platform");
    if (!GemCoordinates.isValidName(name)) {
      throw RubygemsException.badRequest("A valid gem_name is required");
    }
    if (!GemVersion.isValid(version)) {
      throw RubygemsException.badRequest("A valid version is required");
    }
    if (!GemPlatform.isValid(platform)) {
      throw RubygemsException.badRequest("Invalid platform: " + platform);
    }
    GemCoordinates coordinates = new GemCoordinates(name, version.strip(), platform);
    if (!hosted(context).yank(coordinates, path -> authorize(context, HttpMethods.DELETE, path))) {
      throw RubygemsException.notFound("The version " + coordinates.versionAndPlatform() + " of " + name
          + " does not exist.");
    }
    return RubygemsResponses.text(200, "Successfully deleted gem: " + name + " (" + coordinates.versionAndPlatform()
        + ")");
  };

  static void authorize(final Context context, final String action, final String path) {
    Repository repository = context.getRepository();
    Request request = new Request.Builder().action(action).path(path).build();
    repository.facet(SecurityFacet.class).ensurePermitted(request);
  }

  static Map<String, String> form(final Request request) throws IOException {
    Map<String, String> form = new LinkedHashMap<>();
    request.getParameters().forEach(entry -> form.putIfAbsent(entry.getKey(), entry.getValue()));
    Payload payload = request.getPayload();
    if (payload != null && payload.getSize() != 0) {
      String contentType = payload.getContentType();
      if (contentType == null || contentType.startsWith("application/x-www-form-urlencoded")) {
        byte[] body;
        try (InputStream in = payload.openInputStream()) {
          body = in.readNBytes(MAX_FORM_SIZE);
        }
        for (String pair : new String(body, UTF_8).split("&")) {
          if (pair.isEmpty()) {
            continue;
          }
          int equals = pair.indexOf('=');
          String key = URLDecoder.decode(equals < 0 ? pair : pair.substring(0, equals), UTF_8);
          String value = equals < 0 ? "" : URLDecoder.decode(pair.substring(equals + 1), UTF_8);
          form.putIfAbsent(key, value);
        }
      }
    }
    return form;
  }

  private static RubygemsHostedFacet hosted(final Context context) {
    return context.getRepository().facet(RubygemsHostedFacet.class);
  }
}
