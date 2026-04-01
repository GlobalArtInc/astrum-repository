
package org.globalart.astrum.repository.pypi.internal.content;

import java.io.IOException;
import java.io.InputStream;
import java.util.Comparator;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.TreeMap;
import java.util.stream.Collectors;

import javax.annotation.Nullable;

import org.globalart.astrum.common.hash.HashAlgorithm;
import org.globalart.astrum.common.template.TemplateHelper;
import org.globalart.astrum.repository.Facet.Exposed;
import org.globalart.astrum.repository.FacetSupport;
import org.globalart.astrum.repository.content.AssetBlob;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.fluent.FluentAssetBuilder;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.pypi.content.PypiContentFacet;
import org.globalart.astrum.repository.pypi.internal.AssetKind;
import org.globalart.astrum.repository.pypi.internal.PyPiAttributes;
import org.globalart.astrum.repository.pypi.internal.PyPiFormat;
import org.globalart.astrum.repository.pypi.internal.PyPiIndexFacet;
import org.globalart.astrum.repository.pypi.internal.PyPiInfoUtils;
import org.globalart.astrum.repository.pypi.internal.PyPiLink;
import org.globalart.astrum.repository.pypi.internal.SignablePyPiPackage;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.payloads.StringPayload;
import org.globalart.astrum.repository.view.payloads.TempBlob;
import org.globalart.astrum.repository.view.payloads.TempBlobPartPayload;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.entity.Continuations.streamOf;
import static org.globalart.astrum.repository.pypi.internal.AssetKind.ROOT_INDEX;
import static org.globalart.astrum.repository.pypi.internal.PyPiAttributes.P_ASSET_KIND;
import static org.globalart.astrum.repository.pypi.internal.PyPiAttributes.P_NAME;
import static org.globalart.astrum.repository.pypi.internal.PyPiAttributes.P_SUMMARY;
import static org.globalart.astrum.repository.pypi.internal.PyPiAttributes.P_VERSION;
import static org.globalart.astrum.repository.pypi.internal.PyPiIndexUtils.buildIndexPage;
import static org.globalart.astrum.repository.pypi.internal.PyPiIndexUtils.buildRootIndexPage;
import static org.globalart.astrum.repository.pypi.internal.PyPiPathUtils.normalizeName;
import static org.globalart.astrum.repository.pypi.internal.PyPiStorageUtils.mayAddEtag;
import static org.globalart.astrum.repository.pypi.internal.PyPiStorageUtils.validateDigests;
import static org.globalart.astrum.repository.pypi.internal.content.ContentPypiPathUtils.indexPath;
import static org.globalart.astrum.repository.pypi.internal.content.ContentPypiPathUtils.packagesPath;
import static org.globalart.astrum.repository.pypi.internal.content.PyPiDataUtils.copySupportedAttributes;
import static org.globalart.astrum.repository.pypi.internal.content.PyPiDataUtils.getChecksum;
import static org.globalart.astrum.repository.pypi.internal.content.PyPiDataUtils.setFormatAttribute;
import static org.globalart.astrum.repository.view.ContentTypes.TEXT_HTML;

@Exposed
@Component
@Qualifier(PyPiFormat.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class PyPiHostedFacet
    extends FacetSupport
{
  private final TemplateHelper templateHelper;

  @Autowired
  public PyPiHostedFacet(final TemplateHelper templateHelper) {
    this.templateHelper = checkNotNull(templateHelper);
  }

  public FluentAsset upload(final SignablePyPiPackage pyPiPackage) throws IOException {
    Map<String, String> attributes = pyPiPackage.getAttributes();
    try (TempBlobPartPayload wheelPayload = pyPiPackage.getWheelPayload();
         TempBlobPartPayload gpgPayload = pyPiPackage.getGpgSignature()) {
      FluentAsset wheelAsset = savePyPiWheelPayload(wheelPayload.getName(), attributes, wheelPayload);
      if (gpgPayload != null) {
        storeGpgSignaturePayload(gpgPayload, attributes.get(P_NAME), attributes.get(P_VERSION));
      }
      return wheelAsset;
    }
  }

  public FluentAsset upload(
      final String filename,
      final Map<String, String> attributes,
      final TempBlobPartPayload payload)
  {
    return savePyPiWheelPayload(filename, attributes, payload);
  }

  public Content uploadSignature(final String name, final String version, final TempBlobPartPayload payload) {
    return storeGpgSignaturePayload(payload, name, version);
  }

  public String createPackagePath(final String name, final String version, final String filename) {
    return packagesPath(normalizeName(name), version, filename);
  }

  @Nullable
  public Content getPackage(final String packagePath) {
    checkNotNull(packagePath);
    FluentAsset asset = content().getAsset(packagePath).orElse(null);
    if (asset == null) {
      return null;
    }
    AssetBlob blob = asset.blob().orElse(null);
    if (blob == null) {
      return null;
    }
    Content content = asset.download();
    mayAddEtag(content.getAttributes(), blob.checksums().get(HashAlgorithm.SHA1.name()));
    return content;
  }

  public Content getRootIndex() {
    return content().getAsset(indexPath()).orElseGet(this::createRootIndex).download();
  }

  @Nullable
  public Content getIndex(final String name) {
    checkNotNull(name);
    String normalizedName = normalizeName(name);
    PypiContentFacet contentFacet = content();
    if (!contentFacet.isComponentExists(normalizedName)) {
      return null;
    }
    FluentAssetBuilder builder = contentFacet.assets().path(indexPath(normalizedName));
    return builder.find()
        .orElseGet(() -> {
          try (TempBlob tempBlob = contentFacet.getTempBlob(
              new StringPayload(buildIndex(normalizedName), TEXT_HTML))) {
            return builder.kind(AssetKind.INDEX.name()).blob(tempBlob).save();
          }
        })
        .download();
  }

  public Map<String, String> extractMetadata(final TempBlob tempBlob) throws IOException {
    try (InputStream in = tempBlob.get()) {
      return PyPiInfoUtils.extractMetadata(in);
    }
  }

  private FluentAsset createRootIndex() {
    Map<String, PyPiLink> links = new TreeMap<>();
    streamOf(content().components()::browse)
        .forEach(component -> links.put(component.name(), new PyPiLink(component.name(), component.name() + "/")));
    String rootIndexHtml = buildRootIndexPage(templateHelper, links.values());
    try (TempBlob tempBlob = content().getTempBlob(new StringPayload(rootIndexHtml, TEXT_HTML))) {
      return content().assets()
          .path(indexPath())
          .kind(ROOT_INDEX.name())
          .blob(tempBlob)
          .save();
    }
  }

  private String buildIndex(final String name) {
    List<PyPiLink> links = content().assetsByComponentName(name).stream()
        .filter(asset -> AssetKind.PACKAGE.name().equals(asset.kind()))
        .sorted(Comparator.comparing(FluentAsset::path))
        .map(this::buildPyPiLink)
        .flatMap(Optional::stream)
        .collect(Collectors.toList());
    return buildIndexPage(templateHelper, name, links);
  }

  private Optional<PyPiLink> buildPyPiLink(final FluentAsset asset) {
    String uri = asset.path().startsWith("/") ? asset.path().substring(1) : asset.path();
    String file = uri.substring(uri.lastIndexOf('/') + 1);
    Optional<String> sha256 = getChecksum(asset, HashAlgorithm.SHA256);
    String fragment = sha256.map(value -> "#sha256=" + value)
        .orElseGet(() -> getChecksum(asset, HashAlgorithm.MD5).map(value -> "#md5=" + value).orElse(""));
    String requiresPython = asset.attributes(PyPiFormat.NAME).get(PyPiAttributes.P_REQUIRES_PYTHON, String.class, "");
    return Optional.of(new PyPiLink(file, "../../" + uri + fragment, requiresPython));
  }

  private FluentAsset savePyPiWheelPayload(
      final String filename,
      final Map<String, String> attributes,
      final TempBlobPartPayload wheelPayload)
  {
    checkNotNull(filename);
    TempBlob tempBlob = wheelPayload.getTempBlob();
    String name = checkNotNull(attributes.get(P_NAME));
    String version = checkNotNull(attributes.get(P_VERSION));
    String normalizedName = normalizeName(name);
    String packagePath = createPackagePath(name, version, filename);

    validateDigests(attributes, tempBlob);

    PypiContentFacet contentFacet = content();
    PyPiIndexFacet indexFacet = facet(PyPiIndexFacet.class);
    indexFacet.deleteIndex(name);
    if (!contentFacet.isComponentExists(normalizedName)) {
      indexFacet.deleteRootIndex();
    }

    FluentComponent component = contentFacet.findOrCreateComponent(name, version, normalizedName);
    String summary = attributes.get(P_SUMMARY);
    if (summary != null) {
      setFormatAttribute(component, P_SUMMARY, summary);
    }
    setFormatAttribute(component, P_VERSION, version);

    FluentAsset asset = contentFacet.saveAsset(packagePath, component, AssetKind.PACKAGE.name(), tempBlob);
    setFormatAttribute(asset, P_ASSET_KIND, AssetKind.PACKAGE.name());
    copySupportedAttributes(asset, attributes);
    return asset.markAsCached(wheelPayload);
  }

  private Content storeGpgSignaturePayload(
      final TempBlobPartPayload gpgPayload,
      final String name,
      final String version)
  {
    PypiContentFacet contentFacet = content();
    String normalizedName = normalizeName(name);
    FluentComponent component = contentFacet.components().name(normalizedName).version(version).find()
        .orElseGet(() -> {
          facet(PyPiIndexFacet.class).deleteRootIndex();
          return contentFacet.findOrCreateComponent(name, version, normalizedName);
        });
    FluentAsset asset = contentFacet.saveAsset(
        createPackagePath(name, version, gpgPayload.getName()),
        component,
        AssetKind.PACKAGE_SIGNATURE.name(),
        gpgPayload.getTempBlob());
    asset.markAsCached(gpgPayload);
    return asset.download();
  }

  private PypiContentFacet content() {
    return facet(PypiContentFacet.class);
  }
}
