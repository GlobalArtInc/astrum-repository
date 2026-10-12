package org.globalart.astrum.repository.rubygems.internal;

import java.io.InputStream;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.stream.Stream;

import javax.annotation.Nullable;

import org.globalart.astrum.common.hash.HashAlgorithm;
import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.rubygems.AssetKind;
import org.globalart.astrum.repository.rubygems.internal.gem.GemSpec;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.payloads.TempBlob;

import static org.globalart.astrum.common.hash.HashAlgorithm.MD5;
import static org.globalart.astrum.common.hash.HashAlgorithm.SHA1;
import static org.globalart.astrum.common.hash.HashAlgorithm.SHA256;

@Facet.Exposed
public interface RubygemsContentFacet
    extends ContentFacet
{
  String ATTRIBUTES = "rubygems";

  String SIGNATURE = "signature";

  List<HashAlgorithm> HASHING = List.of(MD5, SHA1, SHA256);

  Optional<FluentAsset> getAsset(String path);

  TempBlob ingest(Payload payload);

  TempBlob ingest(InputStream in, @Nullable String contentType);

  FluentAsset putGem(String path, GemSpec spec, TempBlob blob);

  FluentAsset putFile(String path, AssetKind kind, TempBlob blob, Map<String, Object> attributes);

  List<FluentAsset> gems(String name);

  Stream<FluentAsset> allGems();

  boolean delete(String path);
}
