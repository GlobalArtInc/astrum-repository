
package org.globalart.astrum.repository.npm.internal;

import java.io.IOException;
import java.util.List;
import java.util.Objects;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.json.StreamingObjectMapper;
import org.globalart.astrum.repository.json.UntypedObjectDeserializerSerializer;

import com.fasterxml.jackson.core.JsonGenerator;

import static java.util.Collections.emptyList;
import static java.util.Objects.nonNull;
import static java.util.stream.Collectors.toList;
import static org.globalart.astrum.repository.npm.internal.NpmMetadataUtils.META_ID;
import static org.globalart.astrum.repository.npm.internal.NpmMetadataUtils.META_REV;

/**
 * Object mapper implementation for NPM streaming in and out of JSON.
 *
 * @since 3.16
 */
public class NpmStreamingObjectMapper
    extends StreamingObjectMapper
{
  private final String packageId;

  private final String packageRev;

  private List<NpmFieldMatcher> matchers;

  public NpmStreamingObjectMapper() {
    this(null, null, emptyList());
  }

  public NpmStreamingObjectMapper(final List<NpmFieldMatcher> matchers) {
    this(null, null, matchers);
  }

  public NpmStreamingObjectMapper(@Nullable final String packageId,
                                  @Nullable final String packageRev,
                                  final List<NpmFieldMatcher> matchers)
  {
    this.packageId = packageId;
    this.packageRev = packageRev;
    this.matchers = matchers;
  }

  @Override
  protected UntypedObjectDeserializerSerializer untypedObjectDeserializer(final JsonGenerator generator) {
    return new NpmUntypedObjectDeserializerSerializer(generator, matchers);
  }

  @Override
  protected void beforeDeserialize(final JsonGenerator generator) throws IOException {
    super.beforeDeserialize(generator);
    maybeWritePackageAndRevID(generator);
  }

  @Override
  protected void afterDeserialize(final JsonGenerator generator) throws IOException {
    super.afterDeserialize(generator);
    appendFieldsIfNeverMatched(generator);
  }

  private void appendFieldsIfNeverMatched(final JsonGenerator generator) throws IOException {
    for (NpmFieldMatcher matcher : unmatched()) {
      NpmFieldDeserializer deserializer = matcher.getDeserializer();
      generator.writeFieldName(matcher.getFieldName());
      generator.writeObject(deserializer.deserializeValue(null));
    }
  }

  private void maybeWritePackageAndRevID(final JsonGenerator generator) throws IOException {
    if (nonNull(packageId)) {
      generator.writeFieldName(META_ID);
      generator.writeObject(packageId);
    }

    if (nonNull(packageRev)) {
      generator.writeFieldName(META_REV);
      generator.writeObject(packageRev);
    }
  }

  private List<NpmFieldUnmatcher> unmatched() {
    return matchers.stream().map(this::npmFieldUnmatchedFilter)
        .filter(Objects::nonNull)
        .filter(NpmFieldUnmatcher::wasNeverMatched)
        .collect(toList());
  }

  private NpmFieldUnmatcher npmFieldUnmatchedFilter(final NpmFieldMatcher fieldMatcher) {
    return fieldMatcher instanceof NpmFieldUnmatcher ? (NpmFieldUnmatcher) fieldMatcher : null;
  }
}
