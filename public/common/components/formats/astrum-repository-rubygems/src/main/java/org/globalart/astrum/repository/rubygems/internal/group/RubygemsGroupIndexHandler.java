package org.globalart.astrum.repository.rubygems.internal.group;

import java.io.BufferedWriter;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.io.OutputStreamWriter;
import java.io.UncheckedIOException;
import java.io.Writer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.time.Instant;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.HashSet;
import java.util.LinkedHashMap;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Optional;
import java.util.Set;
import java.util.TreeSet;
import java.util.concurrent.locks.Lock;
import java.util.stream.Collectors;

import javax.annotation.Nonnull;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.group.GroupFacet;
import org.globalart.astrum.repository.group.GroupHandler;
import org.globalart.astrum.repository.http.HttpMethods;
import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.rubygems.AssetKind;
import org.globalart.astrum.repository.rubygems.internal.RubygemsContentFacet;
import org.globalart.astrum.repository.rubygems.internal.RubygemsPaths;
import org.globalart.astrum.repository.rubygems.internal.RubygemsResponses;
import org.globalart.astrum.repository.rubygems.internal.index.CompactIndex;
import org.globalart.astrum.repository.rubygems.internal.index.LegacySpecs;
import org.globalart.astrum.repository.rubygems.internal.index.VersionsLine;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.payloads.TempBlob;

import com.google.common.util.concurrent.Striped;
import org.springframework.stereotype.Component;

import static java.nio.charset.StandardCharsets.UTF_8;
import static org.globalart.astrum.repository.rubygems.internal.RubygemsResponses.BINARY;
import static org.globalart.astrum.repository.rubygems.internal.RubygemsResponses.TEXT_UTF8;

@Component
public class RubygemsGroupIndexHandler
    extends GroupHandler
{
  static final int MERGE_FORMAT = 1;

  private final Striped<Lock> locks = Striped.lock(16);

  @Nonnull
  @Override
  protected Response doGet(
      @Nonnull final Context context,
      @Nonnull final DispatchedRepositories dispatched) throws Exception
  {
    String path = context.getRequest().getPath();
    if (RubygemsPaths.VERSIONS.equals(path)) {
      return versions(context, dispatched).map(HttpResponses::ok).orElseGet(() -> notFoundResponse(context));
    }
    if (RubygemsPaths.NAMES.equals(path)) {
      return names(context, dispatched);
    }
    if (path.startsWith(RubygemsPaths.INFO_PREFIX)) {
      return info(context, dispatched);
    }
    LegacySpecs.Kind kind = LegacySpecs.Kind.fromPath(path);
    if (kind != null) {
      return specs(context, dispatched, kind);
    }
    return notFoundResponse(context);
  }

  private Response info(final Context context, final DispatchedRepositories dispatched) throws Exception {
    List<byte[]> infos = new ArrayList<>();
    for (Response response : getAll(get(context.getRequest().getPath()), context, members(context), dispatched)
        .values()) {
      byte[] info = RubygemsResponses.read(response);
      if (info != null) {
        infos.add(info);
      }
    }
    if (infos.isEmpty()) {
      return notFoundResponse(context);
    }
    return HttpResponses.ok(RubygemsResponses.content(CompactIndex.mergeInfo(infos), TEXT_UTF8, null));
  }

  private Response names(final Context context, final DispatchedRepositories dispatched) throws Exception {
    Set<String> names = new LinkedHashSet<>();
    boolean found = false;
    for (Response response : getAll(get(RubygemsPaths.NAMES), context, members(context), dispatched).values()) {
      byte[] body = RubygemsResponses.read(response);
      if (body != null) {
        found = true;
        names.addAll(CompactIndex.parseNames(body));
      }
    }
    if (!found) {
      return notFoundResponse(context);
    }
    return HttpResponses.ok(RubygemsResponses.content(CompactIndex.names(names), TEXT_UTF8, null));
  }

  Optional<Content> versions(final Context context, final DispatchedRepositories dispatched) throws Exception {
    Map<Repository, Content> versions = new LinkedHashMap<>();
    for (Map.Entry<Repository, Response> entry : getAll(get(RubygemsPaths.VERSIONS), context, members(context),
        dispatched).entrySet()) {
      Response response = entry.getValue();
      if (response.getStatus().isSuccessful() && response.getPayload() instanceof Content content) {
        versions.put(entry.getKey(), content);
      }
      else if (response.getPayload() != null) {
        response.getPayload().close();
      }
    }
    if (versions.isEmpty()) {
      return Optional.empty();
    }
    if (versions.size() == 1) {
      return Optional.of(RubygemsResponses.withMd5Etag(versions.values().iterator().next()));
    }
    String signature = signature(versions);
    Repository group = context.getRepository();
    Lock lock = locks.get(group.getName() + RubygemsPaths.VERSIONS);
    lock.lock();
    try {
      Optional<Content> cached = cached(group, RubygemsPaths.VERSIONS, signature);
      if (cached.isPresent()) {
        close(versions.values());
        return cached;
      }
      Path merged = Files.createTempFile("rubygems-versions-", ".txt");
      try {
        mergeVersions(context, versions, merged);
        return Optional.of(RubygemsResponses.withMd5Etag(
            save(group, RubygemsPaths.VERSIONS, merged, TEXT_UTF8, signature)));
      }
      finally {
        close(versions.values());
        Files.deleteIfExists(merged);
      }
    }
    finally {
      lock.unlock();
    }
  }

  private Response specs(
      final Context context,
      final DispatchedRepositories dispatched,
      final LegacySpecs.Kind kind) throws Exception
  {
    Optional<Content> versions = versions(context, dispatched);
    if (versions.isEmpty()) {
      return notFoundResponse(context);
    }
    String signature = MERGE_FORMAT + ":" + RubygemsResponses.md5(versions.get()).orElse("");
    Repository group = context.getRepository();
    Lock lock = locks.get(group.getName() + kind.path());
    lock.lock();
    try {
      Optional<Content> cached = cached(group, kind.path(), signature);
      if (cached.isPresent()) {
        versions.get().close();
        return HttpResponses.ok(cached.get());
      }
      Map<String, String> byName = new LinkedHashMap<>();
      try (Payload payload = versions.get(); InputStream in = payload.openInputStream()) {
        CompactIndex.forEachVersionsLine(in, line -> collect(byName, line));
      }
      Path specs = Files.createTempFile("rubygems-specs-", ".gz");
      try {
        try (OutputStream out = Files.newOutputStream(specs)) {
          LegacySpecs.write(kind, consumer -> byName.forEach((name, joined) ->
              consumer.accept(name, List.of(joined.split(",")))), out);
        }
        return HttpResponses.ok(save(group, kind.path(), specs, BINARY, signature));
      }
      finally {
        Files.deleteIfExists(specs);
      }
    }
    finally {
      lock.unlock();
    }
  }

  static void collect(final Map<String, String> byName, final VersionsLine line) {
    for (String version : line.versions()) {
      if (version.startsWith("-")) {
        String removed = version.substring(1);
        byName.computeIfPresent(line.name(), (name, joined) -> {
          List<String> remaining = new ArrayList<>(List.of(joined.split(",")));
          remaining.remove(removed);
          return remaining.isEmpty() ? null : String.join(",", remaining);
        });
      }
      else if (!version.isEmpty()) {
        byName.merge(line.name(), version, (joined, added) -> joined + "," + added);
      }
    }
  }

  private void mergeVersions(final Context context, final Map<Repository, Content> versions, final Path target)
      throws Exception
  {
    Set<String> all = new HashSet<>();
    Set<String> overlapping = new HashSet<>();
    for (Content content : versions.values()) {
      Set<String> names = new HashSet<>();
      try (InputStream in = content.openInputStream()) {
        CompactIndex.forEachVersionsLine(in, line -> names.add(line.name()));
      }
      for (String name : names) {
        if (!all.add(name)) {
          overlapping.add(name);
        }
      }
    }
    all.clear();
    try (Writer writer = new BufferedWriter(new OutputStreamWriter(Files.newOutputStream(target), UTF_8), 65536)) {
      writer.write(CompactIndex.header(Instant.now()));
      for (Content content : versions.values()) {
        try (InputStream in = content.openInputStream()) {
          CompactIndex.forEachVersionsLine(in, line -> {
            if (!overlapping.contains(line.name())) {
              write(writer, line.toLine());
            }
          });
        }
      }
      List<Repository> members = new ArrayList<>(versions.keySet());
      for (String name : new TreeSet<>(overlapping)) {
        List<byte[]> infos = new ArrayList<>();
        for (Response response : getAll(get(RubygemsPaths.INFO_PREFIX + name), context, members,
            new DispatchedRepositories()).values()) {
          byte[] info = RubygemsResponses.read(response);
          if (info != null) {
            infos.add(info);
          }
        }
        if (infos.isEmpty()) {
          continue;
        }
        byte[] merged = CompactIndex.mergeInfo(infos);
        List<String> mergedVersions = CompactIndex.versionsOf(merged);
        if (!mergedVersions.isEmpty()) {
          write(writer, new VersionsLine(name, mergedVersions, CompactIndex.md5(merged)).toLine());
        }
      }
    }
    log.debug("Merged the versions of {} members into {}, {} gems in more than one member", versions.size(),
        context.getRepository().getName(), overlapping.size());
  }

  private static void write(final Writer writer, final String line) {
    try {
      writer.write(line);
      writer.write('\n');
    }
    catch (IOException e) {
      throw new UncheckedIOException(e);
    }
  }

  private static String signature(final Map<Repository, Content> versions) {
    return versions.entrySet().stream()
        .map(entry -> entry.getKey().getName() + "=" + RubygemsResponses.md5(entry.getValue()).orElse("?"))
        .collect(Collectors.joining(",", MERGE_FORMAT + ":", ""));
  }

  private static Optional<Content> cached(final Repository group, final String path, final String signature) {
    return group.facet(RubygemsContentFacet.class).getAsset(path)
        .filter(asset -> Objects.equals(signature,
            asset.attributes(RubygemsContentFacet.ATTRIBUTES).get(RubygemsContentFacet.SIGNATURE)))
        .map(FluentAsset::download);
  }

  private static Content save(
      final Repository group,
      final String path,
      final Path file,
      final String contentType,
      final String signature) throws IOException
  {
    RubygemsContentFacet content = group.facet(RubygemsContentFacet.class);
    try (InputStream in = Files.newInputStream(file); TempBlob blob = content.ingest(in, contentType)) {
      Map<String, Object> attributes = new HashMap<>();
      attributes.put(RubygemsContentFacet.SIGNATURE, signature);
      return content.putFile(path, AssetKind.INDEX, blob, attributes).download();
    }
  }

  private void close(final Iterable<Content> contents) {
    for (Content content : contents) {
      try {
        content.close();
      }
      catch (IOException e) {
        log.debug("Unable to close member content", e);
      }
    }
  }

  static Request get(final String path) {
    return new Request.Builder().action(HttpMethods.GET).path(path).build();
  }

  private static List<Repository> members(final Context context) {
    return context.getRepository().facet(GroupFacet.class).members();
  }
}
