package org.globalart.astrum.repository.terraform;

import java.util.List;
import java.util.Map;
import java.util.Optional;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.payloads.TempBlob;

@Facet.Exposed
public interface TerraformContentFacet
    extends ContentFacet
{
  String ATTRIBUTES_KEY = TerraformFormat.NAME;

  String KIND = "kind";

  String KIND_MODULE = "module";

  String KIND_PROVIDER = "provider";

  String NAMESPACE = "namespace";

  String NAME = "name";

  String PROVIDER = "provider";

  String VERSION = "version";

  String OS = "os";

  String ARCH = "arch";

  String SUBDIR = "subdir";

  String GENERATED = "generated";

  String PROTOCOLS = "protocols";

  String KEY_ID = "keyId";

  Optional<FluentAsset> getAsset(String path);

  Optional<Content> get(String path);

  TempBlob ingest(Payload payload);

  /**
   * Stores a module package as the asset of its module version component.
   */
  FluentAsset putModule(ModuleCoordinates coordinates, String path, TempBlob blob, Map<String, Object> attributes);

  /**
   * Stores a release file of a provider version as an asset of its component.
   */
  FluentAsset putProviderFile(
      ProviderCoordinates coordinates,
      String fileName,
      AssetKind kind,
      TempBlob blob,
      Map<String, Object> attributes);

  /**
   * Stores a document that belongs to no component, such as a cached registry response.
   */
  FluentAsset putMetadata(String path, AssetKind kind, TempBlob blob);

  boolean delete(String path);

  /**
   * The assets directly below a directory, optionally of one kind.
   */
  List<FluentAsset> assetsIn(String directory, AssetKind... kinds);

  /**
   * The assets anywhere below a directory, optionally of the given kinds.
   */
  List<FluentAsset> assetsUnder(String directory, AssetKind... kinds);
}
