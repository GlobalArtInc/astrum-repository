
package org.globalart.astrum.repository.apt.internal.debian;

import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.Optional;

/**
 * @since 3.17
 */
public class Release
{
  private final ControlFile index;

  public Release(final ControlFile index) {
    super();
    this.index = index;
  }

  public Optional<String> getOrigin() {
    return getValue("Origin");
  }

  public Optional<String> getLabel() {
    return getValue("Label");
  }

  public Optional<String> getSuite() {
    return getValue("Suite");
  }

  public Optional<String> getVersion() {
    return getValue("Version");
  }

  public Optional<String> getCodename() {
    return getValue("Codename");
  }

  public List<String> getComponents() {
    return index.getField("Components").map(s -> s.listValue()).orElse(Collections.emptyList());
  }

  public List<String> getArchitectures() {
    return index.getField("Architectures").map(s -> s.listValue()).orElse(Collections.emptyList());
  }

  public Optional<String> getDescription() {
    return getValue("Description");
  }

  /**
   * Returns the list of relative file paths from the Release file manifest.
   * Tries SHA256, then SHA1, then MD5Sum — all sections list the same paths.
   * Returns an empty list if none of the hash sections are present.
   */
  public List<String> getManifestFiles() {
    String fieldValue = index.getField("SHA256")
        .map(f -> f.value)
        .orElseGet(() -> index.getField("SHA1")
            .map(f -> f.value)
            .orElseGet(() -> index.getField("MD5Sum")
                .map(f -> f.value)
                .orElse("")));

    if (fieldValue.isEmpty()) {
      return Collections.emptyList();
    }

    List<String> paths = new ArrayList<>();
    fieldValue.lines()
        .map(String::trim)
        .filter(line -> !line.isEmpty())
        .map(line -> line.split("\\s+"))
        .filter(parts -> parts.length >= 3)
        .map(parts -> parts[2])
        .forEach(paths::add);
    return paths;
  }

  private Optional<String> getValue(final String name) {
    return index.getField(name).map(e -> e.value);
  }
}
