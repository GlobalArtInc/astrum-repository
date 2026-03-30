
package org.globalart.astrum.repository.npm.internal;

import java.io.IOException;
import java.io.InputStream;
import java.nio.charset.Charset;
import java.util.List;

import javax.annotation.Nullable;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.common.io.InputStreamSupplier;
import org.globalart.astrum.repository.json.MergeObjectMapper;
import org.globalart.astrum.repository.json.NestedAttributesMapJsonParser;
import org.globalart.astrum.repository.json.NestedAttributesMapStdValueInstantiator;
import org.globalart.astrum.repository.json.SourceMapDeserializer;

import com.fasterxml.jackson.databind.DeserializationContext;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.deser.std.MapDeserializer;

import static com.google.common.collect.Maps.newHashMap;
import static java.util.Objects.isNull;
import static java.util.Objects.nonNull;
import static org.globalart.astrum.repository.npm.internal.NpmMetadataUtils.DIST_TAGS;
import static org.globalart.astrum.repository.npm.internal.NpmMetadataUtils.LATEST;
import static org.globalart.astrum.repository.npm.internal.NpmMetadataUtils.META_ID;
import static org.globalart.astrum.repository.npm.internal.NpmMetadataUtils.META_REV;
import static org.globalart.astrum.repository.npm.internal.NpmVersionComparator.versionComparator;

/**
 * {@link ObjectMapper} implementation for NPM Merging (the Astrum way) from {@link InputStream}s
 *
 * @since 3.16
 */
public class NpmMergeObjectMapper
    extends MergeObjectMapper
{
  /**
   * Same as {@link #merge(List, Charset)} but no {@link Charset} needed to be provided.
   *
   * @see #merge(List, Charset)
   */
  @Override
  public NestedAttributesMap merge(final List<InputStreamSupplier> inputStreams) throws IOException {
    return merge(inputStreams, null);
  }

  /**
   * Merge the given {@link InputStream}s into a {@link NestedAttributesMap}. The merging is done according the
   * following rules :
   * <br/>
   * <br/>
   * 1) The order of the {@link InputStream}s dictate which values will be considered the dominant and
   * preserved (the last is the most dominant) value.
   * <br/>
   * 2) The versions in the npm "versions" field will be consolidated together.
   * <br/>
   * 3) We set the "dist-tags/latest" by tracking all versions and comparing which one is the latest.
   * <br/>
   * 4) We remove the "_id" and "_rev" fields as they have no meaning after merge
   *
   * @param inputStreams {@link List} of {@link InputStream}s
   * @param charset      {@link Charset} used for changing from default UTF-8
   * @return NestedAttributesMap
   */
  @Override
  public NestedAttributesMap merge(final List<InputStreamSupplier> inputStreams,
                                   @Nullable final Charset charset)
      throws IOException
  {
    String latestVersion = null;
    NestedAttributesMap result = new NestedAttributesMap("mergeMap", newHashMap());

    for (InputStreamSupplier inputStreamSupplier : inputStreams) {
      try (InputStream inputStream = inputStreamSupplier.get()) {
        merge(result, inputStream, charset);
      }

      latestVersion = updateLatestVersion(result, latestVersion);
    }

    // after all has been parsed mark what we considered the latest version, note that
    // latestVersion might be null leaving a empty dist_tags in the json output
    result.child(DIST_TAGS).set(LATEST, latestVersion);

    // clean up meta id and rev as they have no meaning after merge
    result.remove(META_ID);
    result.remove(META_REV);

    return result;
  }

  @Override
  protected void deserialize(final NestedAttributesMapJsonParser parser,
                             final DeserializationContext context,
                             final MapDeserializer rootDeserializer) throws IOException
  {
    SourceMapDeserializer.of(rootDeserializer.getValueType(),
        new NestedAttributesMapStdValueInstantiator(parser.getRoot()),
        new NpmNestedAttributesMapUntypedObjectDeserializer(parser))
        .deserialize(parser, context);
  }

  private String updateLatestVersion(final NestedAttributesMap result, final String latestVersion) {
    String pkgLatestVersion = result.child(DIST_TAGS).get(LATEST, String.class);

    if (nonNull(pkgLatestVersion) && (isNull(latestVersion) ||
        versionComparator.compare(pkgLatestVersion, latestVersion) > 0)) {
      return pkgLatestVersion;
    }

    return latestVersion;
  }
}
