
package org.globalart.astrum.repository.helm.internal.util;

import org.slf4j.LoggerFactory;
import org.slf4j.Logger;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import java.util.Optional;


import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.matchers.token.TokenMatcher.State;
import org.globalart.astrum.repository.helm.internal.metadata.IndexYamlAbsoluteUrlRewriter;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Utility methods for working with Helm routes and paths.
 *
 * @since 3.28
 */
@Component
public class HelmPathUtils
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final IndexYamlAbsoluteUrlRewriter urlRewriter;

  @Autowired
  public HelmPathUtils(final IndexYamlAbsoluteUrlRewriter urlRewriter) {
    this.urlRewriter = checkNotNull(urlRewriter);
  }

  /**
   * Returns the filename from a {@link State}.
   */
  public String filename(final State state) {
    return match(state, "filename");
  }

  public String contentFilePath(
      final State state,
      final boolean isForwardingSlash)
  {
    String filename = filename(state);
    return isForwardingSlash ? String.format("/%s", filename) : filename;
  }

  public Optional<String> contentFileUrl(
      final String filename,
      final Content indexYaml)
  {
    return urlRewriter.getFirstUrl(indexYaml, filename);
  }

  public String extension(final State state) {
    return match(state, "extension");
  }

  /**
   * Utility method encapsulating getting a particular token by name from a matcher, including preconditions.
   */
  private String match(final State state, final String name) {
    checkNotNull(state);
    String result = state.getTokens().get(name);
    checkNotNull(result);
    return result;
  }

  /**
   * Returns the {@link State} for the content.
   */
  public State matcherState(final Context context) {
    return context.getAttributes().require(State.class);
  }

  public String buildAssetPath(final State matcherState) {
    String filename = filename(matcherState);
    String extension = extension(matcherState);
    return filename + "." + extension;
  }

  public String buildContentAssetPath(final State matcherState) {
    return String.format("/%s", buildAssetPath(matcherState));
  }
}
