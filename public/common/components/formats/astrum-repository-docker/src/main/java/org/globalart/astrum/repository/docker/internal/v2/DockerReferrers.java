package org.globalart.astrum.repository.docker.internal.v2;

import java.io.IOException;
import java.io.InputStream;
import java.net.URLEncoder;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.Collection;
import java.util.Comparator;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

import javax.annotation.Nullable;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.docker.DockerContentFacet;
import org.globalart.astrum.repository.docker.DockerFormat;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Response;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.JsonNode;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * The referrers API: the manifests whose {@code subject} is a digest, as the descriptors of an image index.
 *
 * A hosted repository answers from the manifests it stores and from the index a client without the referrers API
 * kept under the tag of the tag schema; a proxy adds what its remote reports, a group what its members report. The
 * answer is filtered by {@code artifactType} and split into pages of at most the maximum manifest size, linked with
 * {@code Link: <...>; rel="next"}.
 */
public final class DockerReferrers
{
  public static final String ARTIFACT_TYPE_PARAMETER = "artifactType";

  public static final String LAST_PARAMETER = "last";

  public static final String FILTERS_APPLIED_HEADER = "OCI-Filters-Applied";

  private static final Logger log = LoggerFactory.getLogger(DockerReferrers.class);

  private static final TypeReference<Map<String, Object>> DESCRIPTOR = new TypeReference<>() { };

  private static final int PAGE_OVERHEAD = 128;

  private DockerReferrers() {
  }

  /**
   * The referrers a repository stores itself.
   */
  public static List<Map<String, Object>> local(final DockerContentFacet content, final String name,
                                                final String digest)
  {
    List<Map<String, Object>> descriptors = new ArrayList<>();
    for (FluentAsset asset : content.referrers(name, digest)) {
      descriptors.add(descriptor(asset));
    }
    descriptors.addAll(content.fallbackReferrers(name, digest));
    return merge(List.of(descriptors));
  }

  public static Map<String, Object> descriptor(final FluentAsset asset) {
    NestedAttributesMap attributes = asset.attributes(DockerFormat.NAME);
    Map<String, Object> descriptor = new LinkedHashMap<>();
    descriptor.put("mediaType", DockerContentFacet.mediaTypeOf(asset));
    descriptor.put("digest", DockerContentFacet.digestOf(asset));
    descriptor.put("size", asset.blob().map(blob -> blob.blobSize()).orElse(0L));
    Object artifactType = attributes.get(DockerContentFacet.ATTR_ARTIFACT_TYPE);
    if (artifactType != null) {
      descriptor.put("artifactType", artifactType.toString());
    }
    Object annotations = attributes.get(DockerContentFacet.ATTR_ANNOTATIONS);
    if (annotations instanceof Map<?, ?> map && !map.isEmpty()) {
      descriptor.put("annotations", map);
    }
    else if (annotations == null) {
      JsonNode stored = storedAnnotations(asset);
      if (stored != null) {
        descriptor.put("annotations", stored);
      }
    }
    return descriptor;
  }

  @Nullable
  private static JsonNode storedAnnotations(final FluentAsset asset) {
    try (InputStream in = asset.download().openInputStream()) {
      JsonNode annotations = DockerResponses.MAPPER.readTree(in).path("annotations");
      return annotations.isObject() && annotations.size() > 0 ? annotations : null;
    }
    catch (IOException | RuntimeException e) {
      log.debug("Unable to read annotations of {}", asset.path(), e);
      return null;
    }
  }

  /**
   * The descriptors of an image index.
   */
  public static List<Map<String, Object>> parseIndex(final InputStream in) throws IOException {
    List<Map<String, Object>> descriptors = new ArrayList<>();
    JsonNode manifests = DockerResponses.MAPPER.readTree(in).path("manifests");
    for (JsonNode descriptor : manifests) {
      if (descriptor.isObject() && DockerDigest.isDigest(descriptor.path("digest").asText(null))) {
        descriptors.add(DockerResponses.MAPPER.convertValue(descriptor, DESCRIPTOR));
      }
    }
    return descriptors;
  }

  /**
   * The descriptors of all lists, the first one of each digest.
   */
  public static List<Map<String, Object>> merge(final Collection<List<Map<String, Object>>> lists) {
    Map<Object, Map<String, Object>> merged = new LinkedHashMap<>();
    for (List<Map<String, Object>> list : lists) {
      for (Map<String, Object> descriptor : list) {
        Object digest = descriptor.get("digest");
        if (digest != null) {
          merged.putIfAbsent(digest, descriptor);
        }
      }
    }
    return new ArrayList<>(merged.values());
  }

  public static List<Map<String, Object>> filter(
      final List<Map<String, Object>> descriptors,
      @Nullable final String artifactType)
  {
    if (artifactType == null || artifactType.isEmpty()) {
      return descriptors;
    }
    List<Map<String, Object>> filtered = new ArrayList<>();
    for (Map<String, Object> descriptor : descriptors) {
      if (artifactType.equals(descriptor.get("artifactType"))) {
        filtered.add(descriptor);
      }
    }
    return filtered;
  }

  @Nullable
  public static String artifactType(final Context context) {
    String artifactType = context.getRequest().getParameters().get(ARTIFACT_TYPE_PARAMETER);
    return artifactType == null || artifactType.isBlank() ? null : artifactType;
  }

  /**
   * The referrers response: the filtered descriptors in digest order, one page from the {@code last} digest on.
   */
  public static Response response(
      final Context context,
      final String name,
      final String digest,
      final List<Map<String, Object>> descriptors,
      final long maxSize)
  {
    String artifactType = artifactType(context);
    List<Map<String, Object>> sorted = new ArrayList<>(filter(descriptors, artifactType));
    sorted.sort(Comparator.comparing(descriptor -> String.valueOf(descriptor.get("digest"))));
    String last = context.getRequest().getParameters().get(LAST_PARAMETER);
    if (last != null && !last.isEmpty()) {
      sorted.removeIf(descriptor -> String.valueOf(descriptor.get("digest")).compareTo(last) <= 0);
    }

    List<Map<String, Object>> page = new ArrayList<>();
    long size = PAGE_OVERHEAD;
    for (Map<String, Object> descriptor : sorted) {
      long descriptorSize = length(descriptor) + 1;
      if (!page.isEmpty() && size + descriptorSize > maxSize) {
        break;
      }
      page.add(descriptor);
      size += descriptorSize;
    }

    Map<String, Object> body = new LinkedHashMap<>();
    body.put("schemaVersion", 2);
    body.put("mediaType", DockerManifest.OCI_INDEX);
    body.put("manifests", page);
    Response.Builder response = DockerResponses.success(200)
        .payload(DockerResponses.json(body, DockerManifest.OCI_INDEX));
    if (artifactType != null) {
      response.header(FILTERS_APPLIED_HEADER, ARTIFACT_TYPE_PARAMETER);
    }
    if (page.size() < sorted.size()) {
      String next = DockerLocations.referrers(context, name, digest) + "?" + LAST_PARAMETER + "="
          + URLEncoder.encode(String.valueOf(page.get(page.size() - 1).get("digest")), StandardCharsets.UTF_8);
      if (artifactType != null) {
        next += "&" + ARTIFACT_TYPE_PARAMETER + "=" + URLEncoder.encode(artifactType, StandardCharsets.UTF_8);
      }
      response.header("Link", "<" + next + ">; rel=\"next\"");
    }
    return response.build();
  }

  private static long length(final Map<String, Object> descriptor) {
    try {
      return DockerResponses.MAPPER.writeValueAsBytes(descriptor).length;
    }
    catch (JsonProcessingException e) {
      return 0;
    }
  }
}
