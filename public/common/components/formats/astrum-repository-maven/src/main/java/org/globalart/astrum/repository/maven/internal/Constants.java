
package org.globalart.astrum.repository.maven.internal;

import java.util.Locale;

import org.globalart.astrum.repository.view.ContentTypes;

import org.joda.time.format.DateTimeFormat;
import org.joda.time.format.DateTimeFormatter;

/**
 * Maven 2 constants.
 *
 * @since 3.0
 */
public class Constants
{
  private Constants() {
  }

  /**
   * File name of Maven2 repository metadata files.
   */
  public static final String METADATA_FILENAME = "maven-metadata.xml";

  /**
   * File name of Maven2 archetype catalog file.
   */
  public static final String ARCHETYPE_CATALOG_FILENAME = "archetype-catalog.xml";

  /**
   * Layout to be used for a DateTimeFormatter, which maven typically uses in snapshot versions.
   */
  public static final String DOTTED_TIMESTAMP_VERSION_FORMAT = "YYYYMMdd.HHmmss";

  /**
   * {@link DateTimeFormatter} for dotted timestamps used in Maven2 repository metadata.
   */
  public static final DateTimeFormatter METADATA_DOTTED_TIMESTAMP = DateTimeFormat
      .forPattern(DOTTED_TIMESTAMP_VERSION_FORMAT)
      .withZoneUTC()
      .withLocale(Locale.ENGLISH);

  /**
   * {@link DateTimeFormatter} for dotless timestamps used in Maven2 repository metadata.
   */
  public static final DateTimeFormatter METADATA_DOTLESS_TIMESTAMP = DateTimeFormat.forPattern("YYYYMMddHHmmss")
      .withZoneUTC()
      .withLocale(Locale.ENGLISH);

  /**
   * Content Type of Maven2 checksum files (sha1, md5).
   */
  public static final String CHECKSUM_CONTENT_TYPE = ContentTypes.TEXT_PLAIN;

  /**
   * The suffix of base version for snapshots.
   */
  public static final String SNAPSHOT_VERSION_SUFFIX = "SNAPSHOT";

  /**
   * The base path and name of maven index file assets.
   */
  public static final String INDEX_FILE_BASE_PATH = ".index/astrum-maven-repository-index";

  /**
   * The full path of maven index property file assets.
   */
  public static final String INDEX_PROPERTY_FILE_PATH = INDEX_FILE_BASE_PATH + ".properties";

  /**
   * The full path of maven index chunk file assets.
   */
  public static final String INDEX_MAIN_CHUNK_FILE_PATH = INDEX_FILE_BASE_PATH + ".gz";
}
