
package org.globalart.astrum.repository.npm.internal.search.v1;

import org.slf4j.LoggerFactory;
import org.slf4j.Logger;
import org.springframework.stereotype.Component;

import java.io.BufferedInputStream;
import java.io.IOException;
import java.io.InputStream;



import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.datatype.joda.JodaModule;

/**
 * Utility class for abstracting away the actual generation of JSON strings/content from the actual data objects. This
 * class delegates to a Jackson {@code JsonMapper} internally with custom configuration to handle Joda {@code DateTime}
 * objects. Primarily extracted here to make unit testing a bit easier by obviating the need for valid JSON responses.
 *
 * @since 3.7
 */
@Component
public class NpmSearchResponseMapper
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final ObjectMapper mapper;

  public NpmSearchResponseMapper() {
    this.mapper =  new ObjectMapper();
    this.mapper.registerModule(new JodaModule());
  }

  /**
   * Writes the provided {@code NpmSearchResponse} into a string in memory suitable for returning as part of a response.
   * This should be fine for the JSON sizes encountered in search requests (given the typical and maximum size limits).
   */
  public String writeString(final NpmSearchResponse searchResponse) throws JsonProcessingException {
    return mapper.writeValueAsString(searchResponse);
  }

  /**
   * Reads an input stream, marshaling the contents into a {@code NpmSearchResponse} if syntactically valid. Note that
   * this method makes no attempt to ensure that the response is semantically valid, so this must be done by the caller
   * as part of processing the results.
   */
  public NpmSearchResponse readFromInputStream(final InputStream searchResponseStream) throws IOException {
    try (InputStream in = new BufferedInputStream(searchResponseStream)) {
      return mapper.readValue(in, NpmSearchResponse.class);
    }
  }
}
