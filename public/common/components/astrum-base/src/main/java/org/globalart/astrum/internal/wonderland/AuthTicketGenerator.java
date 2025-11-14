
package org.globalart.astrum.internal.wonderland;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.crypto.RandomBytesGenerator;

import com.google.common.io.BaseEncoding;
import org.springframework.beans.factory.annotation.Value;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * Generates random authentication tickets.
 *
 * @since 2.7
 */
@Component
public class AuthTicketGenerator
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private static final String CPREFIX = "${wonderland.authTicketGenerator";

  private final RandomBytesGenerator randomBytes;

  private final int defaultSize;

  // NOTE: Default size is 66 to make full use of base64 encoding w/o padding

  @Autowired
  public AuthTicketGenerator(
      final RandomBytesGenerator randomBytes,
      @Value(CPREFIX + ".defaultSize:66}") final int defaultSize)
  {
    this.randomBytes = checkNotNull(randomBytes);
    this.defaultSize = defaultSize;
    log.debug("Default size: {}", defaultSize);
  }

  protected String encode(final byte[] bytes) {
    return BaseEncoding.base64().encode(bytes);
  }

  public String generate(final int size) {
    byte[] bytes = randomBytes.generate(size);
    return encode(bytes);
  }

  public String generate() {
    return generate(defaultSize);
  }
}
