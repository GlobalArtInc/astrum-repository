
package org.globalart.astrum.repository.composer.internal;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import java.io.IOException;
import java.util.ArrayList;
import java.util.Collection;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

import org.globalart.astrum.repository.view.payloads.TempBlob;

import com.google.common.annotations.VisibleForTesting;
import com.google.common.collect.ImmutableMap;

import static java.util.Objects.requireNonNull;
import static org.globalart.astrum.repository.composer.internal.ComposerAttributes.*;

/**
 * Extracts format attributes from a Composer archive. Currently only zip archives are supported.
 */
@Component
public class ComposerFormatAttributesExtractor
{
  private static final String NAME = "name";

  private static final String DESCRIPTION = "description";

  private static final String VERSION = "version";

  private static final String TYPE = "type";

  private static final String KEYWORDS = "keywords";

  private static final String HOMEPAGE = "homepage";

  private static final String TIME = "time";

  private static final String LICENSE = "license";

  private static final String AUTHORS = "authors";

  private static final String AUTHOR_NAME = "name";

  private static final String AUTHOR_EMAIL = "email";

  private static final String AUTHOR_HOMEPAGE = "homepage";

  private static final String SUPPORT = "support";

  private static final String SUPPORT_EMAIL = "email";

  private static final String SUPPORT_ISSUES = "issues";

  private static final String SUPPORT_FORUM = "forum";

  private static final String SUPPORT_WIKI = "wiki";

  private static final String SUPPORT_IRC = "irc";

  private static final String SUPPORT_SOURCE = "source";

  private static final String SUPPORT_DOCS = "docs";

  private static final String SUPPORT_RSS = "rss";

  private static final Map<String, String> STRINGS_MAPPING = new ImmutableMap.Builder<String, String>()
      .put(NAME, P_NAME)
      .put(DESCRIPTION, P_DESCRIPTION)
      .put(VERSION, P_VERSION)
      .put(TYPE, P_TYPE)
      .put(KEYWORDS, P_KEYWORDS)
      .put(HOMEPAGE, P_HOMEPAGE)
      .put(TIME, P_TIME)
      .put(LICENSE, P_LICENSE)
      .build();

  private static final Map<String, String> SUPPORT_MAPPING = new ImmutableMap.Builder<String, String>()
      .put(SUPPORT_EMAIL, P_SUPPORT_EMAIL)
      .put(SUPPORT_ISSUES, P_SUPPORT_ISSUES)
      .put(SUPPORT_FORUM, P_SUPPORT_FORUM)
      .put(SUPPORT_WIKI, P_SUPPORT_WIKI)
      .put(SUPPORT_IRC, P_SUPPORT_IRC)
      .put(SUPPORT_SOURCE, P_SUPPORT_SOURCE)
      .put(SUPPORT_DOCS, P_SUPPORT_DOCS)
      .put(SUPPORT_RSS, P_SUPPORT_RSS)
      .build();

  private ComposerJsonExtractor composerJsonExtractor;

  @Autowired
  public ComposerFormatAttributesExtractor(final ComposerJsonExtractor composerJsonExtractor) {
    this.composerJsonExtractor = requireNonNull(composerJsonExtractor);
  }

  /**
   * Populates an asset's format attributes with the content contained in a composer.json file in the zip archive. This
   * does not extract all JSON entries, but does try to extract those that could be viewed as more "interesting" from
   * the standpoint of the repository manager.
   */
  public Map<String, Object> extractFromZip(final TempBlob tempBlob) throws IOException {
    Map<String, Object> attributes = new LinkedHashMap<>();
    Map<String, Object> contents = composerJsonExtractor.extractFromZip(tempBlob.getBlob());
    if (!contents.isEmpty()) {
      extractStrings(contents, attributes, STRINGS_MAPPING);
      extractAuthors(contents, attributes);
      extractSupport(contents, attributes);
    }
    return attributes;
  }

  /**
   * Extracts zero or more string-only fields from the source map into the destination attribute map. If a collection
   * is encountered, any string items within the collection are added to a list and stored as a collection of strings.
   */
  @VisibleForTesting
  void extractStrings(final Map<String, Object> source,
                      final Map<String, Object> attributes,
                      final Map<String, String> mappings)
  {
    for (Map.Entry<String, String> mapping : mappings.entrySet()) {
      Object sourceValue = source.get(mapping.getKey());
      if (sourceValue instanceof String) {
        attributes.put(mapping.getValue(), sourceValue);
      }
      else if (sourceValue instanceof Collection) {
        List<String> entries = new ArrayList<>();
        for (Object entryValue : (Collection<?>) sourceValue) {
          if (entryValue instanceof String) {
            entries.add((String) entryValue);
          }
        }
        if (!entries.isEmpty()) {
          attributes.put(mapping.getValue(), entries);
        }
      }
    }
  }

  /**
   * Extracts author contact information (except for the role) into a collection of strings.
   */
  @VisibleForTesting
  void extractAuthors(final Map<String, Object> contents, final Map<String, Object> attributes) {
    Object sourceValue = contents.get(AUTHORS);
    if (sourceValue instanceof Collection) {
      List<String> authors = new ArrayList<>();
      for (Object author : (Collection) sourceValue) {
        if (author instanceof Map) {
          List<String> parts = new ArrayList<>();
          extractAuthorPart((Map<String, Object>) author, parts, AUTHOR_NAME, "%s");
          extractAuthorPart((Map<String, Object>) author, parts, AUTHOR_EMAIL, "<%s>");
          extractAuthorPart((Map<String, Object>) author, parts, AUTHOR_HOMEPAGE, "(%s)");
          if (!parts.isEmpty()) {
            authors.add(String.join(" ", parts));
          }
        }
      }
      if (!authors.isEmpty()) {
        attributes.put(P_AUTHORS, authors);
      }
    }
  }

  /**
   * Extracts one part of the author information into a collection for later joining, applying the specified format
   * string if a string entry with that key is present.
   */
  @VisibleForTesting
  void extractAuthorPart(final Map<String, Object> author,
                         final List<String> parts,
                         final String key,
                         final String format)
  {
    Object part = author.get(key);
    if (part instanceof String) {
      parts.add(String.format(format, part));
    }
  }

  /**
   * Extracts the subkeys for the support entry into their own top-level format attributes.
   */
  private void extractSupport(final Map<String, Object> contents, final Map<String, Object> attributes) {
    Object sourceValue = contents.get(SUPPORT);
    if (sourceValue instanceof Map) {
      extractStrings((Map<String, Object>) sourceValue, attributes, SUPPORT_MAPPING);
    }
  }
}
