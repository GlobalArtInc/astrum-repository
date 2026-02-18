
package org.globalart.astrum.repository.group;

import java.io.IOException;
import java.util.Collection;
import java.util.Map;
import java.util.Optional;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.Response;

/**
 * Merges responses from member repositories.
 */
public abstract class SingleMetadataMergingGroupHandlerSupport
    extends MergingGroupHandlerSupport
{
  @Override
  protected Optional<Content> merge(
      final Context context,
      final Map<Repository, Response> successfulResponses,
      final Optional<String> optEtag) throws IOException
  {
    Optional<Content> result = merge(context, successfulResponses.values()).map(Content::new);

    if (result.isPresent() && optEtag.isPresent()) {
      result.map(Content::getAttributes)
          .ifPresent(attributes -> attributes.set(Content.CONTENT_ETAG, optEtag.get()));

      result = Optional.of(store(context, result.get()));
    }
    else {
      log.debug("Unable to compute lastModified or value");
    }

    return result;
  }

  /**
   * Consume & merge the responses provided to generate the merged metadata.
   *
   * @param context the context of the request
   * @param successfulResponses a list of successful responses to be merged
   * @return a constructed payload, implementors should not persist the value
   */
  protected Optional<Payload> merge(final Context context, final Collection<Response> successfulResponses) {
    return merge(successfulResponses);
  }

  /**
   * Consume & merge the responses provided to generate the merged metadata.
   *
   * @param successfulResponses a list of successful responses to be merged
   * @return a constructed payload, implementors should not persist the value
   */
  protected Optional<Payload> merge(final Collection<Response> successfulResponses) {
    return Optional.empty();
  }

  /**
   * Persist the content merged for the context
   *
   * @param context the request context
   * @param content the content to persist
   * @return the persisted content
   * @throws IOException
   */
  protected abstract Content store(Context context, Content content) throws IOException;
}
