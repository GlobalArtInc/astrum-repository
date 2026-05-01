
package org.globalart.astrum.repository.cocoapods.internal.proxy;

import java.io.IOException;
import java.net.URI;
import java.net.URISyntaxException;
import java.util.Arrays;
import java.util.List;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.cocoapods.internal.PathUtils;
import org.globalart.astrum.repository.cocoapods.internal.git.GitApiHelper;
import org.globalart.astrum.repository.cocoapods.internal.git.GitRepoUriParser;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.ObjectNode;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

@Component
public class SpecFileProcessor
{
  private static final String POD_NAME_FIELD = "name";

  private static final String POD_VERSION_FIELD = "version";

  private static final String SOURCE_NODE_NAME = "source";

  private static final String GIT_NODE_NAME = "git";

  private static final String TAG_NODE_NAME = "tag";

  private static final String COMMIT_NODE_NAME = "commit";

  private static final String BRANCH_NODE_NAME = "branch";

  private static final String SUBMODULES_NODE_NAME = "submodules";

  private static final String HTTP_NODE_NAME = "http";

  private static final List<String> PRESERVED_HTTP_NODE_NAMES = Arrays.asList("type", "flatten", "sha1", "sha256");

  private static final ObjectMapper mapper = new ObjectMapper();

  private final GitApiHelper gitApiHelper;

  @Autowired
  public SpecFileProcessor(final GitApiHelper gitApiHelper) {
    this.gitApiHelper = checkNotNull(gitApiHelper);
  }

  public SpecInfo parse(final String specFile) throws InvalidSpecFileException {
    ObjectNode jsonSpec = readSpec(specFile);
    return new SpecInfo(name(jsonSpec), version(jsonSpec), getExternalURI(jsonSpec.get(SOURCE_NODE_NAME)));
  }

  public String toProxiedSpec(final String specFile, final URI repoUri) throws InvalidSpecFileException {
    ObjectNode jsonSpec = readSpec(specFile);
    String name = name(jsonSpec);
    String version = version(jsonSpec);
    JsonNode originalSource = jsonSpec.get(SOURCE_NODE_NAME);
    URI extDownloadUri = getExternalURI(originalSource);
    if (extDownloadUri == null) {
      return specFile;
    }

    ObjectNode sourceNode = mapper.createObjectNode();
    sourceNode.put(HTTP_NODE_NAME, PathUtils.buildPodUri(repoUri, name, version, extDownloadUri).toString());
    if (originalSource.has(HTTP_NODE_NAME)) {
      for (String preserved : PRESERVED_HTTP_NODE_NAMES) {
        if (originalSource.has(preserved)) {
          sourceNode.set(preserved, originalSource.get(preserved));
        }
      }
    }
    jsonSpec.set(SOURCE_NODE_NAME, sourceNode);
    try {
      return mapper.writerWithDefaultPrettyPrinter().writeValueAsString(jsonSpec);
    }
    catch (IOException e) {
      throw new InvalidSpecFileException("Spec file cannot be written", e);
    }
  }

  private static ObjectNode readSpec(final String specFile) throws InvalidSpecFileException {
    JsonNode node;
    try {
      node = mapper.readTree(specFile);
    }
    catch (IOException e) {
      throw new InvalidSpecFileException("Spec file is not valid JSON", e);
    }
    if (!(node instanceof ObjectNode)) {
      throw new InvalidSpecFileException("Spec file is not a JSON object");
    }
    ObjectNode jsonSpec = (ObjectNode) node;
    if (!jsonSpec.hasNonNull(POD_NAME_FIELD)) {
      throw new InvalidSpecFileException("Spec file without Name");
    }
    if (!jsonSpec.hasNonNull(POD_VERSION_FIELD)) {
      throw new InvalidSpecFileException("Spec file without Version");
    }
    if (!jsonSpec.has(SOURCE_NODE_NAME) || !jsonSpec.get(SOURCE_NODE_NAME).isObject()) {
      throw new InvalidSpecFileException("Spec file without Source");
    }
    return jsonSpec;
  }

  private static String name(final ObjectNode jsonSpec) throws InvalidSpecFileException {
    String name = jsonSpec.get(POD_NAME_FIELD).asText().trim();
    if (!PathUtils.isValidSegment(name)) {
      throw new InvalidSpecFileException("Spec file with invalid Name: " + name);
    }
    return name;
  }

  private static String version(final ObjectNode jsonSpec) throws InvalidSpecFileException {
    String version = jsonSpec.get(POD_VERSION_FIELD).asText().trim();
    if (!PathUtils.isValidSegment(version)) {
      throw new InvalidSpecFileException("Spec file with invalid Version: " + version);
    }
    return version;
  }

  @Nullable
  private URI getExternalURI(final JsonNode sourceNode) {
    if (sourceNode.has(GIT_NODE_NAME)) {
      if (sourceNode.path(SUBMODULES_NODE_NAME).asBoolean(false)) {
        return null;
      }
      URI gitRepoUri = parseGitRepoUri(sourceNode.get(GIT_NODE_NAME).textValue());
      if (gitRepoUri == null) {
        return null;
      }
      try {
        return gitApiHelper.buildDownloadURI(gitRepoUri, extractGitRef(sourceNode));
      }
      catch (IllegalArgumentException e) {
        return null;
      }
    }

    if (sourceNode.has(HTTP_NODE_NAME)) {
      return parseHttpUri(sourceNode.get(HTTP_NODE_NAME).textValue());
    }

    return null;
  }

  @Nullable
  private static String extractGitRef(final JsonNode sourceNode) {
    String[] refNodes = {COMMIT_NODE_NAME, TAG_NODE_NAME, BRANCH_NODE_NAME};
    return Arrays.stream(refNodes)
        .map(sourceNode::get)
        .filter(refNode -> refNode != null && refNode.isValueNode() && !refNode.isNull())
        .map(JsonNode::asText)
        .filter(ref -> !ref.isEmpty())
        .findFirst().orElse(null);
  }

  @Nullable
  private static URI parseHttpUri(@Nullable final String httpUri) {
    if (httpUri == null) {
      return null;
    }
    try {
      URI uri = new URI(httpUri.trim());
      String scheme = uri.getScheme();
      if (uri.getHost() == null || !("http".equalsIgnoreCase(scheme) || "https".equalsIgnoreCase(scheme))) {
        return null;
      }
      return uri;
    }
    catch (URISyntaxException e) {
      return null;
    }
  }

  @Nullable
  private static URI parseGitRepoUri(@Nullable final String gitRepo) {
    if (gitRepo == null) {
      return null;
    }
    URI gitRepoUri;
    try {
      gitRepoUri = new URI(gitRepo.trim());
    }
    catch (URISyntaxException e) {
      return null;
    }

    if (gitRepoUri.getHost() == null || gitRepoUri.getPath() == null) {
      return null;
    }

    if (!GitRepoUriParser.isRepoSupported(gitRepoUri) || !GitRepoUriParser.isGitUriFormatSupported(gitRepoUri)) {
      return null;
    }
    return gitRepoUri;
  }

  public static class SpecInfo
  {
    private final String name;

    private final String version;

    private final URI remoteUri;

    SpecInfo(final String name, final String version, @Nullable final URI remoteUri) {
      this.name = name;
      this.version = version;
      this.remoteUri = remoteUri;
    }

    public String getName() {
      return name;
    }

    public String getVersion() {
      return version;
    }

    @Nullable
    public URI getRemoteUri() {
      return remoteUri;
    }
  }
}
