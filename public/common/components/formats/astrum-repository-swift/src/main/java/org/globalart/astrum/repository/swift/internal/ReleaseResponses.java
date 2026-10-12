package org.globalart.astrum.repository.swift.internal;

import java.util.ArrayList;
import java.util.Base64;
import java.util.List;
import java.util.Map;
import java.util.TreeMap;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.swift.SwiftCoordinates;
import org.globalart.astrum.repository.swift.SwiftVersion;
import org.globalart.astrum.repository.swift.internal.ReleaseMetadata.Signing;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Response;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.node.ObjectNode;
import com.google.common.io.BaseEncoding;

import static org.globalart.astrum.repository.swift.internal.SwiftResponses.CACHE_CONTROL;
import static org.globalart.astrum.repository.swift.internal.SwiftResponses.CONTENT_DISPOSITION;
import static org.globalart.astrum.repository.swift.internal.SwiftResponses.DIGEST;
import static org.globalart.astrum.repository.swift.internal.SwiftResponses.IMMUTABLE;
import static org.globalart.astrum.repository.swift.internal.SwiftResponses.LINK;
import static org.globalart.astrum.repository.swift.internal.SwiftResponses.SIGNATURE;
import static org.globalart.astrum.repository.swift.internal.SwiftResponses.SIGNATURE_FORMAT;

public final class ReleaseResponses
{
  public static final String URL = "url";

  public static final String PROBLEM = "problem";

  public static final String RELEASES = "releases";

  private ReleaseResponses() {
  }

  public static ObjectNode releasesBody(
      final String base,
      final SwiftCoordinates canonical,
      final Map<String, JsonNode> problems)
  {
    ObjectNode body = SwiftResponses.object();
    ObjectNode releases = body.putObject(RELEASES);
    for (String version : sortedDescending(problems.keySet())) {
      ObjectNode release = releases.putObject(version);
      release.put(URL, base + canonical.withVersion(version).getReleasePath());
      JsonNode problem = problems.get(version);
      if (problem != null && !problem.isNull() && !problem.isMissingNode()) {
        release.set(PROBLEM, problem);
      }
    }
    return body;
  }

  public static Response releases(
      final String base,
      final SwiftCoordinates canonical,
      final Map<String, JsonNode> problems,
      final List<Map<String, String>> locations)
  {
    Response response = SwiftResponses.json(releasesBody(base, canonical, problems));
    Links links = new Links();
    List<String> available = new ArrayList<>();
    problems.forEach((version, problem) -> {
      if (problem == null || problem.isNull() || problem.isMissingNode()) {
        available.add(version);
      }
    });
    List<String> sorted = sortedDescending(available);
    if (!sorted.isEmpty()) {
      links.add(base + canonical.withVersion(sorted.get(0)).getReleasePath(), "latest-version");
    }
    for (Map<String, String> location : locations) {
      String rel = location.get("rel");
      if ("canonical".equals(rel) || "alternate".equals(rel)) {
        links.add(location.get(URL), rel);
      }
    }
    setLinks(response, links);
    return response;
  }

  public static Response release(
      final String base,
      final SwiftCoordinates canonical,
      final Content body,
      final List<String> versions)
  {
    Response response = HttpResponses.ok(body);
    setLinks(response, releaseLinks(base, canonical, versions));
    return response;
  }

  public static Links releaseLinks(final String base, final SwiftCoordinates canonical, final List<String> versions) {
    Links links = new Links();
    List<String> sorted = sortedDescending(versions);
    if (sorted.isEmpty()) {
      return links;
    }
    links.add(base + canonical.withVersion(sorted.get(0)).getReleasePath(), "latest-version");
    int index = sorted.indexOf(canonical.getVersion());
    if (index > 0) {
      links.add(base + canonical.withVersion(sorted.get(index - 1)).getReleasePath(), "successor-version");
    }
    if (index >= 0 && index < sorted.size() - 1) {
      links.add(base + canonical.withVersion(sorted.get(index + 1)).getReleasePath(), "predecessor-version");
    }
    return links;
  }

  public static Response manifest(
      final String base,
      final SwiftCoordinates canonical,
      final String filename,
      final Content content,
      final Map<String, String> alternates)
  {
    Response response = HttpResponses.ok(content);
    response.getHeaders().replace(CONTENT_DISPOSITION, SwiftResponses.attachment(filename));
    response.getHeaders().replace(CACHE_CONTROL, IMMUTABLE);
    if (Manifests.MANIFEST.equals(filename)) {
      setLinks(response, manifestLinks(base, canonical, alternates));
    }
    return response;
  }

  public static Links manifestLinks(
      final String base,
      final SwiftCoordinates canonical,
      final Map<String, String> alternates)
  {
    Links links = new Links();
    new TreeMap<>(alternates).forEach((filename, toolsVersion) -> Manifests.variantVersion(filename).ifPresent(
        swiftVersion -> links.addManifest(
            base + canonical.getManifestPath(Manifests.MANIFEST) + "?" + SwiftPaths.SWIFT_VERSION_PARAMETER + "="
                + swiftVersion,
            filename,
            toolsVersion == null || toolsVersion.isBlank() ? swiftVersion : toolsVersion)));
    return links;
  }

  public static Response archive(
      final SwiftCoordinates canonical,
      final Content content,
      @Nullable final String sha256,
      @Nullable final Signing signing)
  {
    Response response = HttpResponses.ok(content);
    response.getHeaders().replace(CONTENT_DISPOSITION, SwiftResponses.attachment(canonical.getArchiveFilename()));
    response.getHeaders().replace(CACHE_CONTROL, IMMUTABLE);
    String digest = digest(sha256);
    if (digest != null) {
      response.getHeaders().replace(DIGEST, digest);
    }
    if (signing != null) {
      response.getHeaders().replace(SIGNATURE_FORMAT, signing.signatureFormat());
      response.getHeaders().replace(SIGNATURE, signing.signatureBase64Encoded());
    }
    return response;
  }

  @Nullable
  public static String digest(@Nullable final String sha256) {
    if (sha256 == null || !sha256.matches("[0-9a-fA-F]{64}")) {
      return null;
    }
    return "sha-256=" + Base64.getEncoder().encodeToString(BaseEncoding.base16().lowerCase()
        .decode(sha256.toLowerCase(java.util.Locale.ROOT)));
  }

  public static List<String> sortedDescending(final Iterable<String> versions) {
    List<String> sorted = new ArrayList<>();
    versions.forEach(sorted::add);
    sorted.sort(SwiftVersion.PRECEDENCE.reversed());
    return sorted;
  }

  private static void setLinks(final Response response, final Links links) {
    if (!links.isEmpty()) {
      response.getHeaders().replace(LINK, links.toString());
    }
  }
}
