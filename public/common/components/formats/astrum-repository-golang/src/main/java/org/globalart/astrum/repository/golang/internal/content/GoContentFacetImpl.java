
package org.globalart.astrum.repository.golang.internal.content;

import java.io.IOException;
import java.io.InputStream;
import java.io.UncheckedIOException;
import java.time.temporal.ChronoUnit;
import java.util.List;
import java.util.Objects;
import java.util.Optional;

import org.globalart.astrum.common.hash.HashAlgorithm;
import org.globalart.astrum.repository.InvalidContentException;
import org.globalart.astrum.repository.config.WritePolicy;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.facet.ContentFacetSupport;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.content.store.FormatStoreManager;
import org.globalart.astrum.repository.golang.AssetKind;
import org.globalart.astrum.repository.golang.GolangFormat;
import org.globalart.astrum.repository.golang.internal.metadata.GolangAttributes;
import org.globalart.astrum.repository.golang.internal.metadata.GolangInfo;
import org.globalart.astrum.repository.golang.internal.util.GoModuleZip;
import org.globalart.astrum.repository.golang.internal.util.GolangPathUtils;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.payloads.BytesPayload;
import org.globalart.astrum.repository.view.payloads.TempBlob;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.lang.String.format;
import static java.nio.charset.StandardCharsets.UTF_8;
import static org.globalart.astrum.common.hash.HashAlgorithm.SHA1;
import static org.globalart.astrum.common.hash.HashAlgorithm.SHA256;
import static org.globalart.astrum.repository.config.WritePolicy.ALLOW_ONCE;
import static org.globalart.astrum.repository.golang.AssetKind.MODULE;
import static org.globalart.astrum.repository.golang.AssetKind.PACKAGE;
import static org.globalart.astrum.repository.golang.internal.util.GolangPathUtils.assetPath;
import static org.globalart.astrum.repository.golang.internal.util.GolangPathUtils.escape;
import static org.globalart.astrum.repository.view.ContentTypes.APPLICATION_JSON;
import static org.globalart.astrum.repository.view.ContentTypes.TEXT_PLAIN;

@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
@Qualifier(GolangFormat.NAME)
public class GoContentFacetImpl
    extends ContentFacetSupport
    implements GoContentFacet
{
  private static final List<HashAlgorithm> HASH_ALGORITHMS = List.of(SHA1, SHA256);

  private static final ObjectMapper MAPPER = new ObjectMapper();

  @Autowired
  public GoContentFacetImpl(@Qualifier(GolangFormat.NAME) final FormatStoreManager formatStoreManager) {
    super(formatStoreManager);
  }

  @Override
  protected WritePolicy writePolicy(final Asset asset) {
    WritePolicy writePolicy = super.writePolicy(asset);
    if (writePolicy == ALLOW_ONCE && !Objects.equals(PACKAGE.name(), asset.kind())) {
      return WritePolicy.ALLOW;
    }
    return writePolicy;
  }

  @Override
  public Optional<FluentAsset> getAsset(final String path) {
    return assets().path(checkNotNull(path)).find();
  }

  @Override
  public Optional<Content> getVersions(final String escapedModule) {
    String module = GolangPathUtils.unescape(checkNotNull(escapedModule));
    if (module == null) {
      return Optional.empty();
    }
    List<String> versions = components().versions("", module).stream()
        .filter(version -> assets().path(assetPath(escapedModule, escape(version), "zip")).find().isPresent())
        .toList();
    if (versions.isEmpty()) {
      return Optional.empty();
    }
    StringBuilder list = new StringBuilder();
    versions.forEach(version -> list.append(version).append('\n'));
    return Optional.of(new Content(new BytesPayload(list.toString().getBytes(UTF_8), TEXT_PLAIN)));
  }

  @Override
  public Optional<Content> getInfo(final GolangAttributes golangAttributes) {
    String zipPath = assetPath(escape(golangAttributes.getModule()), escape(golangAttributes.getVersion()), "zip");
    return assets().path(zipPath)
        .find()
        .flatMap(Asset::blob)
        .map(blob -> info(golangAttributes.getVersion(), blob.blobCreated().toInstant().truncatedTo(ChronoUnit.SECONDS)
            .toString()));
  }

  @Override
  public TempBlob ingest(final Payload payload) {
    return blobs().ingest(checkNotNull(payload), HASH_ALGORITHMS);
  }

  @Override
  public FluentAsset upload(final GolangAttributes golangAttributes, final Payload payload) {
    try (TempBlob zipBlob = ingest(payload)) {
      return upload(golangAttributes, zipBlob);
    }
  }

  @Override
  public FluentAsset upload(final GolangAttributes golangAttributes, final TempBlob zipBlob) {
    checkNotNull(golangAttributes);
    checkNotNull(zipBlob);
    GoModuleZip moduleZip = readModuleZip(zipBlob);
    GolangAttributes attributes = resolve(golangAttributes, moduleZip);
    GolangPathUtils.validate(attributes);

    String module = escape(attributes.getModule());
    String version = escape(attributes.getVersion());
    FluentComponent component = findOrCreateComponent(attributes);
    FluentAsset zipAsset = assets()
        .path(assetPath(module, version, "zip"))
        .kind(PACKAGE.name())
        .component(component)
        .blob(zipBlob)
        .save();

    byte[] goMod = moduleZip.goMod() != null
        ? moduleZip.goMod()
        : ("module " + attributes.getModule() + "\n").getBytes(UTF_8);
    try (TempBlob modBlob = blobs().ingest(new BytesPayload(goMod, TEXT_PLAIN), HASH_ALGORITHMS)) {
      assets()
          .path(assetPath(module, version, "mod"))
          .kind(MODULE.name())
          .component(component)
          .blob(modBlob)
          .save();
    }
    return zipAsset;
  }

  private static GoModuleZip readModuleZip(final TempBlob zipBlob) {
    try (InputStream in = zipBlob.get()) {
      return GoModuleZip.read(in);
    }
    catch (IOException e) {
      throw new UncheckedIOException(e);
    }
  }

  @Override
  public FluentAsset saveComponentAndAsset(
      final String path,
      final Payload payload,
      final AssetKind assetKind,
      final GolangAttributes golangAttributes)
  {
    FluentComponent component = findOrCreateComponent(golangAttributes);
    try (TempBlob tempBlob = blobs().ingest(payload, HASH_ALGORITHMS)) {
      return assets()
          .path(path)
          .kind(assetKind.name())
          .component(component)
          .blob(tempBlob)
          .save();
    }
  }

  @Override
  public FluentAsset saveAsset(final String path, final Payload payload, final AssetKind assetKind) {
    try (TempBlob tempBlob = blobs().ingest(payload, HASH_ALGORITHMS)) {
      return assets()
          .path(path)
          .kind(assetKind.name())
          .blob(tempBlob)
          .save();
    }
  }

  private GolangAttributes resolve(final GolangAttributes requested, final GoModuleZip moduleZip) {
    String module = requested.getModule() == null ? moduleZip.module() : requested.getModule();
    String version = requested.getVersion() == null ? moduleZip.version() : requested.getVersion();
    if (!module.equals(moduleZip.module()) || !version.equals(moduleZip.version())) {
      throw new InvalidContentException(format("Module zip contains %s@%s but %s@%s was expected",
          moduleZip.module(), moduleZip.version(), module, version));
    }
    GolangAttributes attributes = new GolangAttributes();
    attributes.setModule(module);
    attributes.setVersion(version);
    return attributes;
  }

  private FluentComponent findOrCreateComponent(final GolangAttributes golangAttributes) {
    return components()
        .name(checkNotNull(golangAttributes.getModule()))
        .version(checkNotNull(golangAttributes.getVersion()))
        .getOrCreate();
  }

  private Content info(final String version, final String time) {
    try {
      byte[] info = MAPPER.writeValueAsBytes(new GolangInfo(version, time));
      return new Content(new BytesPayload(info, APPLICATION_JSON));
    }
    catch (JsonProcessingException e) {
      throw new UncheckedIOException(e);
    }
  }
}
