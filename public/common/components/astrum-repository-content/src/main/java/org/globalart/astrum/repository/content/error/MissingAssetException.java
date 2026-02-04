
package org.globalart.astrum.repository.content.error;

import java.util.Arrays;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

import static java.lang.String.format;

public class MissingAssetException
    extends RuntimeException
{
  private final List<String> UNSUPPORTED_FORMATS = Arrays.asList("docker");

  private final String assetPath;

  private final String format;

  private final boolean isWritableMember;

  private final String repositoryName;

  private final String componentName;

  private final String message;

  public MissingAssetException(
      final String assetPath,
      final String format,
      final boolean isWritableMember,
      final String repositoryName,
      final String componentName)
  {
    this.assetPath = assetPath;
    this.format = format;
    this.isWritableMember = isWritableMember;
    this.repositoryName = repositoryName;
    this.componentName = componentName;
    this.message = buildMessage();
  }

  private String buildMessage() {
    String message = format("missing asset with path '%s'", assetPath);

    if (isWritableMember && UNSUPPORTED_FORMATS.contains(format)) {
      message = format("%s, %s", message, format(
          "staging moves are not supported for hosted repositories that are configured as the writable member for a %s group repository.",
          format));
    }
    return message;
  }

  @Override
  public String getMessage() {
    return message;
  }

  public String getAssetPath() {
    return assetPath;
  }

  public String getComponentName() {
    return componentName;
  }

  public String getRepositoryName() {
    return repositoryName;
  }

  public Map<String, String> getData() {
    Map<String, String> data = new HashMap<>();
    data.put("path", assetPath);
    data.put("component", componentName);
    data.put("repository", repositoryName);

    return data;
  }
}
