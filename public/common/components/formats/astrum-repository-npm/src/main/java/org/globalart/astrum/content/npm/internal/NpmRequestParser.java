
package org.globalart.astrum.content.npm.internal;

import org.slf4j.LoggerFactory;
import org.slf4j.Logger;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import java.io.IOException;
import java.io.InputStreamReader;
import java.nio.charset.Charset;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.InvalidContentException;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.content.npm.NpmContentFacet;
import org.globalart.astrum.repository.npm.internal.NpmAttributes.AssetKind;
import org.globalart.astrum.repository.npm.internal.NpmPublishParser;
import org.globalart.astrum.repository.npm.internal.NpmPublishRequest;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.payloads.TempBlob;
import org.globalart.astrum.security.SecuritySystem;
import org.globalart.astrum.security.user.User;
import org.globalart.astrum.security.user.UserNotFoundException;

import com.fasterxml.jackson.core.JsonFactory;
import com.fasterxml.jackson.core.JsonParseException;
import com.fasterxml.jackson.core.JsonParser;
import com.google.common.annotations.VisibleForTesting;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.nio.charset.StandardCharsets.ISO_8859_1;
import static java.nio.charset.StandardCharsets.UTF_8;
import static org.globalart.astrum.content.npm.internal.NpmContentFacetImpl.HASHING;

/**
 * Component for parsing various kinds of incoming npm requests, performing special optimizations as possible for the
 * nature of the request.
 *
 * This class should remain rather "thin" and focus on bookkeeping matters related to request parsing. As the special
 * handling for incoming requests may be different depending on the optimizations (attachment fields, etc.) custom
 * behavior should reside in specific parser classes.
 *
 * @since 3.4
 */
@Component
public class NpmRequestParser
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private static final JsonFactory jsonFactory = new JsonFactory();

  private final SecuritySystem securitySystem;

  @Autowired
  public NpmRequestParser(final SecuritySystem securitySystem) {
    this.securitySystem = securitySystem;
  }

  /**
   * Parses an incoming "npm publish" or "npm unpublish" request, returning the results. Note that you should probably
   * call this from within a try-with-resources block to manage the lifecycle of any temp blobs created during the
   * operation.
   */
  public NpmPublishRequest parsePublish(final Repository repository, final Payload payload) throws IOException {
    checkNotNull(repository);
    checkNotNull(payload);

    NpmContentFacet content = repository.facet(NpmContentFacet.class);
    try (TempBlob tempBlob = content.blobs().ingest(payload, HASHING)) {
      try {
        return parseNpmPublish(content, tempBlob, UTF_8);
      }
      catch (JsonParseException e) {
        // fallback
        if (e.getMessage().contains("Invalid UTF-8")) {
          // try again, but assume ISO8859-1 encoding now, that is illegal for JSON
          return parseNpmPublish(content, tempBlob, ISO_8859_1);
        }
        throw new InvalidContentException("Invalid JSON input", e);
      }
    }
  }

  @VisibleForTesting
  NpmPublishRequest parseNpmPublish(
      final NpmContentFacet content,
      final TempBlob tempBlob,
      final Charset charset) throws IOException
  {
    try (InputStreamReader reader = new InputStreamReader(tempBlob.get(), charset)) {
      try (JsonParser jsonParser = jsonFactory.createParser(reader)) {
        NpmPublishParser parser = npmPublishParserFor(jsonParser, content);
        return parser.parse(getUserId());
      }
    }
  }

  @Nullable
  private String getUserId() {
    try {
      User currentUser = securitySystem.currentUser();
      return currentUser == null ? null : currentUser.getUserId();
    }
    catch (UserNotFoundException e) { // NOSONAR
      log.debug("No user found, no name replacement will occur");
      return null;
    }
  }

  @VisibleForTesting
  NpmPublishParser npmPublishParserFor(final JsonParser jsonParser, final NpmContentFacet content) {
    return new NpmPublishParser(jsonParser,
        (in, hashAlgs) -> content.blobs().ingest(in, AssetKind.TARBALL.getContentType(), hashAlgs), HASHING);
  }
}
