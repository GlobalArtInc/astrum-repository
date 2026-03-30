
package org.globalart.astrum.repository.npm.internal;

import java.io.IOException;

import com.fasterxml.jackson.core.JsonParser;

/**
 * A NPM Json Field Matcher the test whether we never matched a field.
 *
 * @since 3.17
 */
public class NpmFieldUnmatcher
    extends NpmFieldMatcher
{
  private int matched = 0;

  public NpmFieldUnmatcher(final String fieldName,
                           final String pathRegex,
                           final NpmFieldDeserializer deserializer)
  {
    super(fieldName, pathRegex, deserializer);
  }

  @Override
  public boolean allowDeserializationOnMatched() {
    return false;
  }

  /**
   * Test for matches using the super class and marks if it was matched.
   *
   * @see NpmFieldMatcher#matches(JsonParser)
   */
  @Override
  public boolean matches(final JsonParser parser) throws IOException {
    boolean matches = super.matches(parser);

    if (matched == 0 && matches) {
      matched = 1;
    }

    return matches;
  }

  /**
   * Test whether at the current parsed state of a {@link NpmFieldMatcher} it was matched
   * by <code>fieldName</code> and <code>pathRegex</code>.
   */
  public boolean wasNeverMatched() {
    return matched == 0;
  }
}
