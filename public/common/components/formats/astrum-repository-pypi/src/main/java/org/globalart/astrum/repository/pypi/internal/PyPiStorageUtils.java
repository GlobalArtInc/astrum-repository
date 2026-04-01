
package org.globalart.astrum.repository.pypi.internal;

import java.io.IOException;
import java.io.InputStream;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

import org.globalart.astrum.common.collect.AttributesMap;
import org.globalart.astrum.common.hash.HashAlgorithm;
import org.globalart.astrum.repository.IllegalOperationException;
import org.globalart.astrum.repository.view.PartPayload;
import org.globalart.astrum.repository.view.payloads.TempBlob;

import com.google.common.collect.ImmutableList;
import com.google.common.collect.ImmutableMap;
import com.google.common.hash.HashCode;
import org.apache.commons.lang3.StringUtils;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.nio.charset.StandardCharsets.UTF_8;
import static org.globalart.astrum.common.hash.HashAlgorithm.MD5;
import static org.globalart.astrum.common.hash.HashAlgorithm.SHA1;
import static org.globalart.astrum.common.hash.HashAlgorithm.SHA256;
import static org.globalart.astrum.repository.pypi.internal.PyPiAttributes.P_NAME;
import static org.globalart.astrum.repository.pypi.internal.PyPiPathUtils.normalizeName;
import static org.globalart.astrum.repository.view.Content.CONTENT_ETAG;

public class PyPiStorageUtils
{
  public static final List<HashAlgorithm> HASH_ALGORITHMS = ImmutableList.of(SHA1, SHA256, MD5);

  private static final Map<String, HashAlgorithm> DIGEST_ATTRIBUTES =
      ImmutableMap.of("md5_digest", MD5, "sha256_digest", SHA256);

  private PyPiStorageUtils() {
  }

  public static void validateDigests(final Map<String, String> attributes, final TempBlob tempBlob) {
    for (Map.Entry<String, HashAlgorithm> entry : DIGEST_ATTRIBUTES.entrySet()) {
      String expectedDigest = attributes.get(entry.getKey());
      if (StringUtils.isBlank(expectedDigest)) {
        continue;
      }
      HashCode hashCode = tempBlob.getHashes().get(entry.getValue());
      if (hashCode == null) {
        continue;
      }
      String hashValue = hashCode.toString();
      if (!expectedDigest.trim().equalsIgnoreCase(hashValue)) {
        throw new IllegalOperationException(
            "Digests do not match, found: " + hashValue + ", expected: " + expectedDigest);
      }
    }
  }

  public static void addAttribute(final Map<String, String> attributes, final PartPayload payload) throws IOException {
    checkNotNull(attributes);
    checkNotNull(payload);
    try (InputStream in = payload.openInputStream()) {
      String fieldName = payload.getFieldName();
      String newValue = new String(in.readAllBytes(), UTF_8);
      String oldValue = attributes.get(fieldName);
      if (StringUtils.isNotBlank(oldValue)) {
        newValue = oldValue + "\n" + newValue;
      }
      attributes.put(fieldName, newValue);
    }
  }

  public static void mayAddEtag(final AttributesMap attributesMap, final String hashCode) {
    if (attributesMap.contains(CONTENT_ETAG)) {
      return;
    }
    if (hashCode != null) {
      attributesMap.set(CONTENT_ETAG, "{SHA1{" + hashCode + "}}");
    }
  }

  public static Map<String, String> getNameAttributes(final String name) {
    Map<String, String> nameAttributes = new HashMap<>();
    nameAttributes.put(P_NAME, name);
    nameAttributes.put("name_dash", normalizeName(name, "-"));
    nameAttributes.put("name_dot", normalizeName(name, "."));
    nameAttributes.put("name_underscore", normalizeName(name, "_"));
    return nameAttributes;
  }
}
