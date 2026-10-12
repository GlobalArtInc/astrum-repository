package org.globalart.astrum.repository.huggingface.internal;

import java.util.Optional;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.view.Payload;

@Facet.Exposed
public interface HuggingFaceContentFacet
    extends ContentFacet
{
  Optional<FluentAsset> getAsset(String path);

  /**
   * Stores the information of a revision, a JSON document without a component.
   */
  FluentAsset saveRevision(String path, byte[] json);

  /**
   * Stores a file of a repository as an asset of the component {@code <repo type>/<repo id>@<commit>}.
   *
   * @throws org.globalart.astrum.repository.InvalidContentException when the content does not have the SHA-256 or
   *           the size the Hub announced
   */
  FluentAsset saveFile(HuggingFaceFile file, Payload content);

  /**
   * Stores a file without fetching it when the same path of the repository is stored at another commit with the
   * ETag the Hub announced for it: weights rarely change between commits.
   */
  Optional<FluentAsset> reuseFile(HuggingFaceFile file);
}
