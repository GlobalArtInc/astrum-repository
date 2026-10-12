package org.globalart.astrum.repository.rubygems.internal.hosted;

import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.time.Instant;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.TreeMap;
import java.util.concurrent.locks.ReentrantLock;
import java.util.function.Consumer;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.FacetSupport;
import org.globalart.astrum.repository.InvalidContentException;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.maintenance.ContentMaintenanceFacet;
import org.globalart.astrum.repository.rubygems.AssetKind;
import org.globalart.astrum.repository.rubygems.RubygemsFormat;
import org.globalart.astrum.repository.rubygems.internal.RubygemsContentFacet;
import org.globalart.astrum.repository.rubygems.internal.RubygemsResponses;
import org.globalart.astrum.repository.rubygems.internal.gem.GemCoordinates;
import org.globalart.astrum.repository.rubygems.internal.gem.GemPackage;
import org.globalart.astrum.repository.rubygems.internal.gem.GemSpec;
import org.globalart.astrum.repository.rubygems.internal.index.CompactIndex;
import org.globalart.astrum.repository.rubygems.internal.index.InfoEntry;
import org.globalart.astrum.repository.rubygems.internal.index.LegacySpecs;
import org.globalart.astrum.repository.rubygems.internal.index.QuickSpec;
import org.globalart.astrum.repository.rubygems.internal.index.VersionsLine;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.payloads.TempBlob;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.rubygems.internal.RubygemsResponses.BINARY;
import static org.globalart.astrum.repository.rubygems.internal.RubygemsResponses.TEXT_UTF8;

@Facet.Exposed
@Component
@Qualifier(RubygemsFormat.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class RubygemsHostedFacet
    extends FacetSupport
{
  public static final String VERSIONS = "/versions";

  public static final String NAMES = "/names";

  static final String INDEX_FORMAT = "indexFormat";

  static final int CURRENT_INDEX_FORMAT = 1;

  private final ReentrantLock indexLock = new ReentrantLock();

  public FluentAsset push(final Payload payload, final Consumer<String> authorizer) throws IOException {
    RubygemsContentFacet content = content();
    try (TempBlob blob = content.ingest(payload)) {
      GemSpec spec = parse(blob);
      GemCoordinates coordinates = spec.coordinates();
      authorizer.accept(coordinates.gemPath());
      FluentAsset asset = content.putGem(coordinates.gemPath(), spec, blob);
      log.info("Pushed gem {} to {}", coordinates, getRepository().getName());
      invalidateIndexes();
      return asset;
    }
  }

  public boolean yank(final GemCoordinates coordinates, final Consumer<String> authorizer) {
    Optional<FluentAsset> asset = content().getAsset(coordinates.gemPath());
    if (asset.isEmpty()) {
      return false;
    }
    authorizer.accept(coordinates.gemPath());
    facet(ContentMaintenanceFacet.class).deleteAsset(asset.get());
    log.info("Yanked gem {} from {}", coordinates, getRepository().getName());
    invalidateIndexes();
    return true;
  }

  public Optional<Content> gem(final String path) {
    return content().getAsset(path).map(FluentAsset::download);
  }

  public Optional<Content> info(final String name) {
    List<InfoEntry> entries = new ArrayList<>();
    for (FluentAsset asset : content().gems(name)) {
      InfoEntry entry = infoEntry(asset);
      if (entry != null) {
        entries.add(entry);
      }
    }
    if (entries.isEmpty()) {
      return Optional.empty();
    }
    return Optional.of(RubygemsResponses.content(CompactIndex.info(entries), TEXT_UTF8, null));
  }

  public Optional<Content> quickSpec(final String fullName) {
    return content().getAsset(GemCoordinates.gemPathForFullName(fullName))
        .map(RubygemsHostedFacet::spec)
        .map(spec -> RubygemsResponses.content(QuickSpec.deflated(spec), BINARY, null));
  }

  public Content versions() throws IOException {
    return RubygemsResponses.withMd5Etag(index(VERSIONS));
  }

  public Content names() throws IOException {
    return RubygemsResponses.withMd5Etag(index(NAMES));
  }

  public Content specs(final LegacySpecs.Kind kind) throws IOException {
    return index(kind.path());
  }

  public void invalidateIndexes() {
    indexLock.lock();
    try {
      RubygemsContentFacet content = content();
      content.delete(VERSIONS);
      content.delete(NAMES);
      for (LegacySpecs.Kind kind : LegacySpecs.Kind.values()) {
        content.delete(kind.path());
      }
    }
    finally {
      indexLock.unlock();
    }
  }

  private Content index(final String path) throws IOException {
    Optional<FluentAsset> cached = current(path);
    if (cached.isPresent()) {
      return cached.get().download();
    }
    indexLock.lock();
    try {
      cached = current(path);
      if (cached.isEmpty()) {
        rebuildIndexes();
        cached = content().getAsset(path);
      }
      return cached.orElseThrow(() -> new IllegalStateException("Index " + path + " was not built")).download();
    }
    finally {
      indexLock.unlock();
    }
  }

  private Optional<FluentAsset> current(final String path) {
    return content().getAsset(path).filter(asset ->
        asset.attributes(RubygemsContentFacet.ATTRIBUTES).get(INDEX_FORMAT) instanceof Number format
            && format.intValue() == CURRENT_INDEX_FORMAT);
  }

  private void rebuildIndexes() throws IOException {
    Map<String, List<InfoEntry>> entries = new TreeMap<>();
    content().allGems().forEach(asset -> {
      InfoEntry entry = infoEntry(asset);
      if (entry != null) {
        entries.computeIfAbsent(spec(asset).name(), key -> new ArrayList<>()).add(entry);
      }
    });
    List<VersionsLine> lines = new ArrayList<>();
    Map<String, List<String>> versions = new TreeMap<>();
    entries.forEach((name, list) -> {
      byte[] info = CompactIndex.info(list);
      List<String> sorted = list.stream().sorted(InfoEntry.ORDER).map(InfoEntry::versionAndPlatform).toList();
      versions.put(name, sorted);
      lines.add(new VersionsLine(name, sorted, CompactIndex.md5(info)));
    });
    save(VERSIONS, CompactIndex.versions(Instant.now(), lines));
    save(NAMES, CompactIndex.names(entries.keySet()));
    for (LegacySpecs.Kind kind : LegacySpecs.Kind.values()) {
      ByteArrayOutputStream out = new ByteArrayOutputStream();
      LegacySpecs.write(kind, versions::forEach, out);
      save(kind.path(), out.toByteArray());
    }
    log.debug("Rebuilt the indexes of {} with {} gems", getRepository().getName(), entries.size());
  }

  private void save(final String path, final byte[] bytes) {
    String contentType = path.endsWith(".gz") ? BINARY : TEXT_UTF8;
    try (TempBlob blob = content().ingest(new ByteArrayInputStream(bytes), contentType)) {
      content().putFile(path, AssetKind.INDEX, blob, Map.of(INDEX_FORMAT, CURRENT_INDEX_FORMAT));
    }
  }

  @Nullable
  private static InfoEntry infoEntry(final FluentAsset asset) {
    Map<String, Object> attributes = asset.attributes(RubygemsContentFacet.ATTRIBUTES).backing();
    if (attributes.get(GemSpec.NAME) == null || attributes.get(GemSpec.VERSION) == null) {
      return null;
    }
    String checksum = asset.blob().map(blob -> blob.checksums().get("sha256")).orElse("");
    return InfoEntry.of(GemSpec.fromAttributes(attributes), checksum);
  }

  private static GemSpec spec(final FluentAsset asset) {
    return GemSpec.fromAttributes(asset.attributes(RubygemsContentFacet.ATTRIBUTES).backing());
  }

  public static GemSpec parse(final TempBlob blob) {
    try (InputStream in = blob.get()) {
      return GemPackage.readSpec(in);
    }
    catch (IllegalArgumentException e) {
      throw new InvalidContentException(e.getMessage());
    }
    catch (IOException e) {
      throw new InvalidContentException("Not a gem file: " + e.getMessage());
    }
  }

  private RubygemsContentFacet content() {
    return facet(RubygemsContentFacet.class);
  }
}
