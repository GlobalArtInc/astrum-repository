
package org.globalart.astrum.repository.conan;

import java.util.List;
import java.util.Map;
import java.util.Optional;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.conan.internal.ConanInfo;
import org.globalart.astrum.repository.conan.internal.ConanReferencePattern;
import org.globalart.astrum.repository.conan.internal.ConanRevision;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.payloads.TempBlob;

@Facet.Exposed
public interface ConanContentFacet
    extends ContentFacet
{
  Optional<FluentAsset> getAsset(String path);

  TempBlob ingest(Payload payload);

  FluentAsset put(ConanCoords coords, String path, AssetKind kind, TempBlob blob);

  FluentAsset put(ConanCoords coords, String path, AssetKind kind, Payload payload);

  List<ConanRevision> recipeRevisions(ConanCoords coords);

  Optional<ConanRevision> latestRecipeRevision(ConanCoords coords);

  boolean recipeRevisionExists(ConanCoords coords);

  List<ConanRevision> packageRevisions(ConanCoords coords);

  Optional<ConanRevision> latestPackageRevision(ConanCoords coords);

  boolean packageRevisionExists(ConanCoords coords);

  Map<String, FluentAsset> files(String directory);

  Map<String, ConanInfo> packageInfos(ConanCoords coords);

  List<ConanCoords> searchRecipes(ConanReferencePattern pattern);

  int deleteUnder(ConanCoords recipe, String prefix);
}
